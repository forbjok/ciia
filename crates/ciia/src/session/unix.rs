use dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply;

pub struct SessionInstance(Option<dbus::blocking::Connection>);

pub fn session() -> anyhow::Result<Option<SessionInstance>> {
    // Try to connect to D-Bus to check whether an instance is already running in this session.
    let instance = if let Ok(dbus_session_conn) = dbus::blocking::Connection::new_session() {
        if let Ok(reply) = dbus_session_conn.request_name("org.forbsoft.CIIA", false, false, true) {
            if reply == RequestNameReply::Exists {
                return Ok(None);
            }
        }

        Some(SessionInstance(Some(dbus_session_conn)))
    } else {
        Some(SessionInstance(None))
    };

    Ok(instance)
}
