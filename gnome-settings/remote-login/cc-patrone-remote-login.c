/* -*- mode: C; c-file-style: "gnu"; indent-tabs-mode: nil; -*- */
/*
 * Patrone Remote Desktop in GNOME Settings: the first tab of System >
 * Remote Desktop, named "Remote Desktop", in place of GNOME's own Remote
 * Login, which drives gnome-remote-desktop's system daemon instead. One
 * switch, "Remote Desktop", turns it on for the account using Settings,
 * through patrone-remote-desktop's helper (/usr/libexec/patrone-remote-
 * desktop/settings, run with pkexec: turning it on or off asks for an
 * administrator's password). Then the page asks for the account's own
 * password once, which the helper checks and records what NLA checks, and
 * shows what to connect to: the computer's name and addresses, the
 * username, and a Remote Desktop Connection file (patrone-remote-desktop
 * rdp-file). Under that it says what the other ways in do: Desktop
 * Sharing (the next tab), Secure Shell and GNOME's Remote Login.
 * billsecond/patrone-remote-desktop's README has the rest.
 *
 * Copyright (C) 2026 William Daugherty
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, see <http://www.gnu.org/licenses/>.
 */

#undef G_LOG_DOMAIN
#define G_LOG_DOMAIN "cc-patrone-remote-login"

#include "config.h"

#include <arpa/inet.h>
#include <ifaddrs.h>
#include <net/if.h>
#include <netinet/in.h>
#include <string.h>

#include <glib/gi18n.h>

#include "cc-patrone-remote-login.h"
#include "secure-shell/cc-secure-shell-page.h"

#define PKEXEC "/usr/bin/pkexec"
#define HELPER "/usr/libexec/patrone-remote-desktop/settings"
#define COMMAND "/usr/sbin/patrone-remote-desktop"
#define SSH_SERVER "/usr/sbin/sshd"

/* pkexec's exit status when the password dialog was dismissed */
#define PKEXEC_DISMISSED 126

struct _CcPatroneRemoteLogin
{
  AdwBin parent_instance;

  AdwToastOverlay *toasts;
  AdwSwitchRow *switch_row;
  GtkWidget *password_group;
  GtkWidget *confirm_button;
  GtkWidget *connect_group;
  char *host;

  GCancellable *cancellable;
  gboolean known;    /* the helper said how things stand */
  gboolean busy;     /* a helper call is running */
  gboolean on;       /* Remote Desktop is on */
  gboolean member;   /* this account may sign in */
  gboolean password; /* NLA can check its password */
  gboolean others;   /* other accounts may sign in */
};

G_DEFINE_TYPE (CcPatroneRemoteLogin, cc_patrone_remote_login, ADW_TYPE_BIN)

typedef void (*HelperDone) (CcPatroneRemoteLogin *self,
                            int                   status,
                            const char           *output,
                            const char           *errors);

typedef struct
{
  CcPatroneRemoteLogin *self;
  HelperDone done;
} HelperCall;

static void on_switch_active (CcPatroneRemoteLogin *self);
static void ask_password (CcPatroneRemoteLogin *self, const char *problem);

static void
add_toast (CcPatroneRemoteLogin *self, const char *message)
{
  adw_toast_overlay_add_toast (self->toasts, adw_toast_new (message));
}

static void
on_helper_done (GObject *source, GAsyncResult *result, gpointer user_data)
{
  g_autofree HelperCall *call = user_data;
  GSubprocess *process = G_SUBPROCESS (source);
  g_autoptr(GError) error = NULL;
  g_autofree char *output = NULL;
  g_autofree char *errors = NULL;
  int status = -1;

  if (!g_subprocess_communicate_utf8_finish (process, result, &output, &errors, &error))
    {
      /* The page is gone */
      if (g_error_matches (error, G_IO_ERROR, G_IO_ERROR_CANCELLED))
        return;
      call->done (call->self, -1, NULL, error->message);
      return;
    }
  if (g_subprocess_get_if_exited (process))
    status = g_subprocess_get_exit_status (process);
  call->done (call->self, status, output, errors);
}

/* Runs one of the helper's commands through pkexec, with INPUT on its
 * stdin, and DONE with its exit status and output */
static void
run_helper (CcPatroneRemoteLogin *self,
            const char           *command,
            const char           *input,
            HelperDone            done)
{
  g_autoptr(GSubprocess) process = NULL;
  g_autoptr(GError) error = NULL;
  HelperCall *call;

  process = g_subprocess_new (G_SUBPROCESS_FLAGS_STDIN_PIPE |
                              G_SUBPROCESS_FLAGS_STDOUT_PIPE |
                              G_SUBPROCESS_FLAGS_STDERR_PIPE,
                              &error, PKEXEC, HELPER, command, NULL);
  if (process == NULL)
    {
      done (self, -1, NULL, error->message);
      return;
    }
  call = g_new0 (HelperCall, 1);
  call->self = self;
  call->done = done;
  g_subprocess_communicate_utf8_async (process, input, self->cancellable, on_helper_done, call);
}

static gboolean
is_yes (char **lines, const char *key)
{
  g_autofree char *line = g_strdup_printf ("%s=yes", key);

  return g_strv_contains ((const char * const *) lines, line);
}

/* The helper's on=, member=, password= and others= lines */
static void
read_status (CcPatroneRemoteLogin *self, const char *output)
{
  g_auto(GStrv) lines = g_strsplit (output != NULL ? output : "", "\n", -1);

  self->known = g_strv_contains ((const char * const *) lines, "on=yes") ||
                g_strv_contains ((const char * const *) lines, "on=no");
  self->on = is_yes (lines, "on");
  self->member = is_yes (lines, "member");
  self->password = is_yes (lines, "password");
  self->others = is_yes (lines, "others");
}

static void
sync_page (CcPatroneRemoteLogin *self)
{
  gboolean active = self->on && self->member;
  const char *subtitle = NULL;

  g_signal_handlers_block_by_func (self->switch_row, on_switch_active, self);
  adw_switch_row_set_active (self->switch_row, active);
  g_signal_handlers_unblock_by_func (self->switch_row, on_switch_active, self);
  gtk_widget_set_sensitive (GTK_WIDGET (self->switch_row), self->known && !self->busy);

  if (!self->known && !self->busy)
    subtitle = _("Couldn't tell whether Remote Desktop is on");
  else if (self->on && !self->member)
    subtitle = _("On for other accounts. Turn it on to sign in with yours too.");
  else if (active && self->others)
    subtitle = _("Turning it off turns it off for every account");
  adw_action_row_set_subtitle (ADW_ACTION_ROW (self->switch_row), subtitle != NULL ? subtitle : "");

  gtk_widget_set_visible (self->password_group, self->known && active && !self->password);
  gtk_widget_set_sensitive (self->confirm_button, !self->busy);
  gtk_widget_set_visible (self->connect_group, self->known && active);
}

static void
on_status (CcPatroneRemoteLogin *self, int status, const char *output, const char *errors)
{
  self->busy = FALSE;
  if (status == 0)
    read_status (self, output);
  else
    {
      g_warning ("Patrone Remote Desktop's status failed (%d): %s", status, errors);
      self->known = FALSE;
    }
  sync_page (self);
}

static void
refresh (CcPatroneRemoteLogin *self)
{
  if (self->busy)
    return;
  self->busy = TRUE;
  sync_page (self);
  run_helper (self, "status", NULL, on_status);
}

static void
on_changed (CcPatroneRemoteLogin *self,
            int                   status,
            const char           *output,
            const char           *errors,
            gboolean              enabling)
{
  self->busy = FALSE;
  if (status != 0)
    {
      if (status != PKEXEC_DISMISSED)
        {
          g_warning ("Patrone Remote Desktop's settings helper failed (%d): %s", status, errors);
          add_toast (self, enabling ? _("Couldn't turn on Remote Desktop")
                                    : _("Couldn't turn off Remote Desktop"));
        }
      /* The switch goes back to how things are */
      refresh (self);
      return;
    }

  read_status (self, output);
  sync_page (self);
  if (enabling && self->on && self->member && !self->password)
    ask_password (self, NULL);
}

static void
on_enabled (CcPatroneRemoteLogin *self, int status, const char *output, const char *errors)
{
  on_changed (self, status, output, errors, TRUE);
}

static void
on_disabled (CcPatroneRemoteLogin *self, int status, const char *output, const char *errors)
{
  on_changed (self, status, output, errors, FALSE);
}

static void
on_switch_active (CcPatroneRemoteLogin *self)
{
  gboolean active = adw_switch_row_get_active (self->switch_row);

  self->busy = TRUE;
  gtk_widget_set_sensitive (GTK_WIDGET (self->switch_row), FALSE);
  run_helper (self, active ? "enable" : "disable", NULL, active ? on_enabled : on_disabled);
}

static void
on_password_checked (CcPatroneRemoteLogin *self, int status, const char *output, const char *errors)
{
  g_autofree char *answer = g_strstrip (g_strdup (output != NULL ? output : ""));

  self->busy = FALSE;
  if (status == 0)
    {
      self->password = TRUE;
      sync_page (self);
      add_toast (self, _("Remote Desktop is ready"));
      return;
    }
  sync_page (self);
  /* The helper answers "denied" for a wrong password */
  if (status == 1 && g_strcmp0 (answer, "denied") == 0)
    ask_password (self, _("That password isn't right. Try again."));
  else
    {
      g_warning ("Patrone Remote Desktop's password check failed (%d): %s", status, errors);
      add_toast (self, _("Couldn't check your password"));
    }
}

static void
on_password_response (AdwAlertDialog       *dialog,
                      const char           *response,
                      CcPatroneRemoteLogin *self)
{
  GtkEditable *entry = g_object_get_data (G_OBJECT (dialog), "entry");
  g_autofree char *password = NULL;

  if (g_strcmp0 (response, "confirm") != 0)
    return;
  password = g_strdup (gtk_editable_get_text (entry));
  gtk_editable_set_text (entry, "");

  self->busy = TRUE;
  sync_page (self);
  run_helper (self, "password", password, on_password_checked);
  explicit_bzero (password, strlen (password));
}

static void
on_password_changed (GtkEditable *entry, AdwAlertDialog *dialog)
{
  adw_alert_dialog_set_response_enabled (dialog, "confirm", gtk_editable_get_text (entry)[0] != '\0');
}

/* Remote Desktop checks the account's password at every connection, against
 * what a password sign-in recorded, so the account gives it once here */
static void
ask_password (CcPatroneRemoteLogin *self, const char *problem)
{
  AdwDialog *dialog = adw_alert_dialog_new (_("Confirm Your Password"), NULL);
  AdwAlertDialog *alert = ADW_ALERT_DIALOG (dialog);
  GtkWidget *entry = gtk_password_entry_new ();

  adw_alert_dialog_set_body (alert, problem != NULL ? problem :
                             _("Remote Desktop Connection asks for your username and password. "
                               "Enter your password here once, so this computer can check it "
                               "when you connect."));
  adw_alert_dialog_add_responses (alert,
                                  "later", _("_Not Now"),
                                  "confirm", _("_Confirm"),
                                  NULL);
  adw_alert_dialog_set_response_appearance (alert, "confirm", ADW_RESPONSE_SUGGESTED);
  adw_alert_dialog_set_default_response (alert, "confirm");
  adw_alert_dialog_set_close_response (alert, "later");
  adw_alert_dialog_set_response_enabled (alert, "confirm", FALSE);

  gtk_password_entry_set_show_peek_icon (GTK_PASSWORD_ENTRY (entry), TRUE);
  g_object_set (entry, "activates-default", TRUE, NULL);
  gtk_accessible_update_property (GTK_ACCESSIBLE (entry),
                                  GTK_ACCESSIBLE_PROPERTY_LABEL, _("Password"),
                                  -1);
  g_signal_connect (entry, "changed", G_CALLBACK (on_password_changed), alert);
  adw_alert_dialog_set_extra_child (alert, entry);
  g_object_set_data (G_OBJECT (dialog), "entry", entry);
  g_signal_connect_object (dialog, "response", G_CALLBACK (on_password_response), self, 0);

  adw_dialog_set_focus (dialog, entry);
  adw_dialog_present (dialog, GTK_WIDGET (self));
}

static void
on_confirm_clicked (CcPatroneRemoteLogin *self)
{
  ask_password (self, NULL);
}

static void
on_copy_clicked (GtkButton *button, CcPatroneRemoteLogin *self)
{
  AdwActionRow *row = g_object_get_data (G_OBJECT (button), "row");

  gdk_clipboard_set_text (gtk_widget_get_clipboard (GTK_WIDGET (button)),
                          adw_action_row_get_subtitle (row));
  add_toast (self, g_object_get_data (G_OBJECT (button), "copied"));
}

/* A value to type into the client, with a button that copies it */
static void
add_value (CcPatroneRemoteLogin *self,
           AdwPreferencesGroup  *group,
           const char           *title,
           const char           *value,
           const char           *copied)
{
  GtkWidget *row = adw_action_row_new ();
  GtkWidget *button = gtk_button_new_from_icon_name ("edit-copy-symbolic");

  adw_preferences_row_set_title (ADW_PREFERENCES_ROW (row), title);
  adw_action_row_set_subtitle (ADW_ACTION_ROW (row), value);
  adw_action_row_set_subtitle_selectable (ADW_ACTION_ROW (row), TRUE);
  gtk_widget_add_css_class (row, "property");

  gtk_widget_set_tooltip_text (button, _("Copy"));
  gtk_widget_set_valign (button, GTK_ALIGN_CENTER);
  gtk_widget_add_css_class (button, "flat");
  g_object_set_data (G_OBJECT (button), "row", row);
  g_object_set_data (G_OBJECT (button), "copied", (gpointer) copied);
  g_signal_connect (button, "clicked", G_CALLBACK (on_copy_clicked), self);
  adw_action_row_add_suffix (ADW_ACTION_ROW (row), button);

  adw_preferences_group_add (group, row);
}

/* Bridges for virtual machines and containers, which other computers can't
 * reach */
static gboolean
is_local_bridge (const char *name)
{
  static const char *const prefixes[] = { "docker", "virbr", "lxdbr", "veth", "br-" };

  for (guint i = 0; i < G_N_ELEMENTS (prefixes); i++)
    if (g_str_has_prefix (name, prefixes[i]))
      return TRUE;
  return FALSE;
}

/* This computer's IPv4 addresses on the networks it's connected to */
static GPtrArray *
find_addresses (void)
{
  GPtrArray *addresses = g_ptr_array_new_with_free_func (g_free);
  struct ifaddrs *list = NULL;

  if (getifaddrs (&list) != 0)
    return addresses;
  for (struct ifaddrs *item = list; item != NULL; item = item->ifa_next)
    {
      const struct sockaddr_in *address = (const struct sockaddr_in *) item->ifa_addr;
      char text[INET_ADDRSTRLEN];

      if (address == NULL || address->sin_family != AF_INET)
        continue;
      if (!(item->ifa_flags & IFF_UP) || !(item->ifa_flags & IFF_RUNNING) ||
          (item->ifa_flags & IFF_LOOPBACK) || is_local_bridge (item->ifa_name))
        continue;
      /* 169.254.x.x: no network gave it one */
      if ((ntohl (address->sin_addr.s_addr) >> 16) == 0xa9fe)
        continue;
      if (inet_ntop (AF_INET, &address->sin_addr, text, sizeof (text)) != NULL &&
          !g_ptr_array_find_with_equal_func (addresses, text, g_str_equal, NULL))
        g_ptr_array_add (addresses, g_strdup (text));
    }
  freeifaddrs (list);
  return addresses;
}

static void
on_saved (GObject *source, GAsyncResult *result, gpointer user_data)
{
  CcPatroneRemoteLogin *self = user_data;
  g_autoptr(GError) error = NULL;
  g_autofree char *name = NULL;
  g_autofree char *message = NULL;

  if (!g_file_replace_contents_finish (G_FILE (source), result, NULL, &error))
    {
      if (g_error_matches (error, G_IO_ERROR, G_IO_ERROR_CANCELLED))
        return;
      g_warning ("Couldn't save the Remote Desktop connection file: %s", error->message);
      add_toast (self, _("Couldn't save the connection file"));
      return;
    }
  name = g_file_get_basename (G_FILE (source));
  message = g_strdup_printf (_("Saved %s"), name);
  add_toast (self, message);
}

static void
on_save_chosen (GObject *source, GAsyncResult *result, gpointer user_data)
{
  CcPatroneRemoteLogin *self = user_data;
  g_autoptr(GFile) file = NULL;
  g_autoptr(GSubprocess) process = NULL;
  g_autoptr(GBytes) contents = NULL;
  g_autoptr(GError) error = NULL;

  file = gtk_file_dialog_save_finish (GTK_FILE_DIALOG (source), result, &error);
  /* Cancelled with the page, or dismissed */
  if (file == NULL)
    return;

  /* patrone-remote-desktop writes the file, as on the command line */
  process = g_subprocess_new (G_SUBPROCESS_FLAGS_STDOUT_PIPE, &error,
                              COMMAND, "rdp-file", self->host, NULL);
  if (process == NULL ||
      !g_subprocess_communicate (process, NULL, self->cancellable, &contents, NULL, &error) ||
      !g_subprocess_get_successful (process))
    {
      g_warning ("patrone-remote-desktop rdp-file failed: %s",
                 error != NULL ? error->message : "it exited with an error");
      add_toast (self, _("Couldn't save the connection file"));
      return;
    }
  g_file_replace_contents_bytes_async (file, contents, NULL, FALSE,
                                       G_FILE_CREATE_REPLACE_DESTINATION,
                                       self->cancellable, on_saved, self);
}

static void
on_save_clicked (CcPatroneRemoteLogin *self)
{
  g_autoptr(GtkFileDialog) dialog = gtk_file_dialog_new ();
  g_autofree char *name = g_strdup_printf ("%s.rdp", g_get_host_name ());

  gtk_file_dialog_set_title (dialog, _("Save Connection File"));
  gtk_file_dialog_set_initial_name (dialog, name);
  gtk_file_dialog_save (dialog, GTK_WINDOW (gtk_widget_get_root (GTK_WIDGET (self))),
                        self->cancellable, on_save_chosen, self);
}

/* A row that says what another way in does */
static GtkWidget *
add_explanation (AdwPreferencesGroup *group, const char *title, const char *text)
{
  GtkWidget *row = adw_action_row_new ();

  adw_preferences_row_set_title (ADW_PREFERENCES_ROW (row), title);
  adw_action_row_set_subtitle (ADW_ACTION_ROW (row), text);
  adw_preferences_group_add (group, row);
  return row;
}

/* ...which opens that way's own settings */
static void
make_link (GtkWidget *row, GCallback callback, gpointer data)
{
  adw_action_row_add_suffix (ADW_ACTION_ROW (row), gtk_image_new_from_icon_name ("go-next-symbolic"));
  gtk_list_box_row_set_activatable (GTK_LIST_BOX_ROW (row), TRUE);
  g_signal_connect_swapped (row, "activated", callback, data);
}

/* Desktop Sharing is the Remote Desktop page's next tab (named so by
 * cc-remote-desktop-page.c in Patrone's build) */
static void
on_desktop_sharing_activated (CcPatroneRemoteLogin *self)
{
  GtkWidget *stack = gtk_widget_get_ancestor (GTK_WIDGET (self), ADW_TYPE_VIEW_STACK);

  if (stack != NULL &&
      adw_view_stack_get_child_by_name (ADW_VIEW_STACK (stack), CC_PATRONE_DESKTOP_SHARING_TAB) != NULL)
    adw_view_stack_set_visible_child_name (ADW_VIEW_STACK (stack), CC_PATRONE_DESKTOP_SHARING_TAB);
}

/* GNOME's own Secure Shell dialog, as System's Secure Shell row opens it */
static void
on_secure_shell_activated (CcPatroneRemoteLogin *self)
{
  adw_dialog_present (ADW_DIALOG (g_object_new (CC_TYPE_SECURE_SHELL_PAGE, NULL)), GTK_WIDGET (self));
}

static GtkWidget *
add_group (AdwPreferencesPage *page, const char *title, const char *description)
{
  AdwPreferencesGroup *group = ADW_PREFERENCES_GROUP (adw_preferences_group_new ());

  if (title != NULL)
    adw_preferences_group_set_title (group, title);
  if (description != NULL)
    adw_preferences_group_set_description (group, description);
  adw_preferences_page_add (page, group);
  return GTK_WIDGET (group);
}

static void
cc_patrone_remote_login_dispose (GObject *object)
{
  CcPatroneRemoteLogin *self = CC_PATRONE_REMOTE_LOGIN (object);

  g_cancellable_cancel (self->cancellable);
  g_clear_object (&self->cancellable);
  g_clear_pointer (&self->host, g_free);

  G_OBJECT_CLASS (cc_patrone_remote_login_parent_class)->dispose (object);
}

static void
cc_patrone_remote_login_class_init (CcPatroneRemoteLoginClass *klass)
{
  GObjectClass *object_class = G_OBJECT_CLASS (klass);

  object_class->dispose = cc_patrone_remote_login_dispose;
}

static void
cc_patrone_remote_login_init (CcPatroneRemoteLogin *self)
{
  AdwPreferencesPage *page = ADW_PREFERENCES_PAGE (adw_preferences_page_new ());
  g_autoptr(GPtrArray) addresses = find_addresses ();
  g_autofree char *ssh_command = NULL;
  g_autofree char *ssh_text = NULL;
  GtkWidget *group, *label, *row, *button;
  const char *host = g_get_host_name ();

  self->cancellable = g_cancellable_new ();
  /* As patrone-remote-desktop's connection file has it: found on the
   * network by its name (mDNS), unless it already has a domain */
  self->host = strchr (host, '.') != NULL ? g_strdup (host) : g_strdup_printf ("%s.local", host);

  group = add_group (page, NULL, NULL);
  label = gtk_label_new (_("Use this computer from another one, with Windows' Remote Desktop "
                           "Connection or another RDP app. You sign in with your own username "
                           "and password and get your own desktop, with your windows as you left "
                           "them. While you're connected, this computer's own screen shows the "
                           "login screen."));
  gtk_label_set_wrap (GTK_LABEL (label), TRUE);
  gtk_label_set_justify (GTK_LABEL (label), GTK_JUSTIFY_CENTER);
  gtk_widget_set_hexpand (label, TRUE);
  gtk_widget_add_css_class (label, "dim-label");
  adw_preferences_group_add (ADW_PREFERENCES_GROUP (group), label);

  group = add_group (page, NULL, NULL);
  self->switch_row = ADW_SWITCH_ROW (adw_switch_row_new ());
  adw_preferences_row_set_title (ADW_PREFERENCES_ROW (self->switch_row), _("_Remote Desktop"));
  adw_preferences_row_set_use_underline (ADW_PREFERENCES_ROW (self->switch_row), TRUE);
  g_signal_connect_swapped (self->switch_row, "notify::active", G_CALLBACK (on_switch_active), self);
  adw_preferences_group_add (ADW_PREFERENCES_GROUP (group), GTK_WIDGET (self->switch_row));

  self->password_group = add_group (page, NULL, NULL);
  row = adw_action_row_new ();
  adw_preferences_row_set_title (ADW_PREFERENCES_ROW (row), _("Confirm Your Password"));
  adw_action_row_set_subtitle (ADW_ACTION_ROW (row),
                               _("Needed once before you can connect. Signing in to this "
                                 "computer with your password does it too."));
  self->confirm_button = gtk_button_new_with_mnemonic (_("_Confirm…"));
  gtk_widget_set_valign (self->confirm_button, GTK_ALIGN_CENTER);
  g_signal_connect_swapped (self->confirm_button, "clicked", G_CALLBACK (on_confirm_clicked), self);
  adw_action_row_add_suffix (ADW_ACTION_ROW (row), self->confirm_button);
  adw_preferences_group_add (ADW_PREFERENCES_GROUP (self->password_group), row);

  group = add_group (page, _("How to Connect"),
                     _("In Remote Desktop Connection, connect to this computer's name or "
                       "address, then sign in with your username and password. The first "
                       "time, it asks you to trust this computer's certificate."));
  self->connect_group = group;
  add_value (self, ADW_PREFERENCES_GROUP (group), _("Computer Name"), self->host,
             _("Computer name copied to clipboard"));
  for (guint i = 0; i < addresses->len; i++)
    add_value (self, ADW_PREFERENCES_GROUP (group), _("IP Address"),
               g_ptr_array_index (addresses, i), _("IP address copied to clipboard"));
  add_value (self, ADW_PREFERENCES_GROUP (group), _("Username"), g_get_user_name (),
             _("Username copied to clipboard"));

  row = adw_action_row_new ();
  adw_preferences_row_set_title (ADW_PREFERENCES_ROW (row), _("Connection File"));
  adw_action_row_set_subtitle (ADW_ACTION_ROW (row),
                               _("Opens Remote Desktop Connection with this computer and your "
                                 "username filled in"));
  button = gtk_button_new_with_mnemonic (_("_Save…"));
  gtk_widget_set_valign (button, GTK_ALIGN_CENTER);
  g_signal_connect_swapped (button, "clicked", G_CALLBACK (on_save_clicked), self);
  adw_action_row_add_suffix (ADW_ACTION_ROW (row), button);
  adw_preferences_group_add (ADW_PREFERENCES_GROUP (group), row);

  /* What the other remote settings do, so nobody has to guess */
  group = add_group (page, _("Other Ways to Connect"), NULL);
  row = add_explanation (ADW_PREFERENCES_GROUP (group), _("Desktop Sharing"),
                         _("Shows the desktop you're using now in another computer's remote "
                           "desktop app, for someone to watch or, with Remote Control on, to "
                           "use with you. It has its own username and password, set on its "
                           "tab, and works only while you're signed in here."));
  make_link (row, G_CALLBACK (on_desktop_sharing_activated), self);

  ssh_command = g_strdup_printf ("ssh %s@%s", g_get_user_name (), self->host);
  if (g_file_test (SSH_SERVER, G_FILE_TEST_IS_EXECUTABLE))
    ssh_text = g_strdup_printf (_("Lets you run commands on this computer from another "
                                  "computer's terminal (%s), signing in with your own "
                                  "username and password. Text only, no desktop."),
                                ssh_command);
  else
    ssh_text = g_strdup_printf (_("Lets you run commands on this computer from another "
                                  "computer's terminal (%s), signing in with your own "
                                  "username and password. Text only, no desktop. It needs "
                                  "the openssh-server package, which isn't installed."),
                                ssh_command);
  row = add_explanation (ADW_PREFERENCES_GROUP (group), _("Secure Shell (SSH)"), ssh_text);
  if (g_file_test (SSH_SERVER, G_FILE_TEST_IS_EXECUTABLE))
    make_link (row, G_CALLBACK (on_secure_shell_activated), self);

  add_explanation (ADW_PREFERENCES_GROUP (group), _("GNOME Remote Login"),
                   _("GNOME's own way to sign in from another computer, which Remote Desktop "
                     "takes the place of here: it opens a login screen, after a username and "
                     "password set for the whole computer. Only one of the two can be on, "
                     "since both use port 3389."));

  self->toasts = ADW_TOAST_OVERLAY (adw_toast_overlay_new ());
  adw_toast_overlay_set_child (self->toasts, GTK_WIDGET (page));
  adw_bin_set_child (ADW_BIN (self), GTK_WIDGET (self->toasts));

  refresh (self);
  /* And each time the page shows again: Remote Desktop may have been turned
   * on or off elsewhere */
  g_signal_connect_swapped (self, "map", G_CALLBACK (refresh), self);
}

/* Where patrone-remote-desktop is installed, it takes Remote Login's place */
gboolean
cc_patrone_remote_login_available (void)
{
  return g_file_test (HELPER, G_FILE_TEST_IS_EXECUTABLE) &&
         g_file_test (PKEXEC, G_FILE_TEST_IS_EXECUTABLE);
}

GtkWidget *
cc_patrone_remote_login_new (void)
{
  return g_object_new (CC_TYPE_PATRONE_REMOTE_LOGIN, NULL);
}
