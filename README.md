# nu_plugin_dbus

## Install with Cargo

Run: `cargo install --locked nu_plugin_dbus`

Then add `register ~/.cargo/bin/nu_plugin_dbus` to your `~/.config/nushell/config.nu`

## Usage

    Commands for interacting with D-Bus

    Search terms: dbus

    Usage:
      > dbus 

    Subcommands:
      dbus call - Call a method and get its response
      dbus get - Get a D-Bus property
      dbus get-all - Get all D-Bus property for the given objects
      dbus introspect - Introspect a D-Bus object
      dbus list - List all available connection names on the bus
      dbus set - Get all D-Bus property for the given objects

    Flags:
      -h, --help - Display the help message for this command

## `dbus introspect`

    Introspect a D-Bus object

    Returns information about available nodes, interfaces, methods, signals, and properties on the given object path

    Search terms: dbus

    Usage:
      > dbus introspect {flags} <object> 

    Flags:
      -h, --help - Display the help message for this command
      --session - Send to the session message bus (default)
      --system - Send to the system message bus
      --started - Send to the bus that started this process, if applicable
      --bus <String> - Send to the bus server at the given address
      --peer <String> - Send to a non-bus D-Bus server at the given address. Will not call the Hello method on initialization.
      --timeout <Duration> - How long to wait for a response
      --dest (required parameter) <String> - The name of the connection that owns the object

    Parameters:
      object <string>: The path to the object to introspect

    Examples:
      Look at the MPRIS2 interfaces exposed by Spotify
      > dbus introspect --dest=org.mpris.MediaPlayer2.spotify /org/mpris/MediaPlayer2 | explore

      Get methods exposed by KDE Plasma's on-screen display service
      > dbus introspect --dest=org.kde.plasmashell /org/kde/osdService | get interfaces | where name == org.kde.osdService | get 0.methods

      List objects exposed by KWin
      > dbus introspect --dest=org.kde.KWin / | get children | select name

## `dbus call`

    Call a method and get its response

    Returns an array if the method call returns more than one value.

    Search terms: dbus

    Usage:
      > dbus call {flags} <object> <interface> <method> ...(args) 

    Flags:
      -h, --help - Display the help message for this command
      --session - Send to the session message bus (default)
      --system - Send to the system message bus
      --started - Send to the bus that started this process, if applicable
      --bus <String> - Send to the bus server at the given address
      --peer <String> - Send to a non-bus D-Bus server at the given address. Will not call the Hello method on initialization.
      --timeout <Duration> - How long to wait for a response
      --signature <String> - Signature of the arguments to send, in D-Bus format.
        If not provided, they will be determined from introspection.
        If --no-introspect is specified and this is not provided, they will be guessed (poorly)
      --no-flatten - Always return a list of all return values
      --no-introspect - Don't use introspection to determine the correct argument signature
      --dest (required parameter) <String> - The name of the connection to send the method to

    Parameters:
      object <string>: The path to the object to call the method on
      interface <string>: The name of the interface the method belongs to
      method <string>: The name of the method to send
      ...args <any>: Arguments to send with the method call

    Examples:
      Ping the D-Bus server itself
      > dbus call --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.Peer Ping

      Show a notification on the desktop for 5 seconds
      > dbus call --dest=org.freedesktop.Notifications /org/freedesktop/Notifications org.freedesktop.Notifications Notify "Floppy disks" 0 "media-floppy" "Rarely seen" "But sometimes still used" [] {} 5000

## `dbus get`

    Get a D-Bus property

    Search terms: dbus

    Usage:
      > dbus get {flags} <object> <interface> <property> 

    Flags:
      -h, --help - Display the help message for this command
      --session - Send to the session message bus (default)
      --system - Send to the system message bus
      --started - Send to the bus that started this process, if applicable
      --bus <String> - Send to the bus server at the given address
      --peer <String> - Send to a non-bus D-Bus server at the given address. Will not call the Hello method on initialization.
      --timeout <Duration> - How long to wait for a response
      --dest (required parameter) <String> - The name of the connection to read the property from

    Parameters:
      object <string>: The path to the object to read the property from
      interface <string>: The name of the interface the property belongs to
      property <string>: The name of the property to read

    Examples:
      Get the currently playing song in Spotify
      > dbus get --dest=org.mpris.MediaPlayer2.spotify /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Metadata
      ╭──────────────┬───────────────────────────────────────────────────────╮
      │ xesam:title  │ Birdie                                                │
      │ xesam:artist │ [list 1 item]                                         │
      │ xesam:album  │ Love Your Love                                        │
      │ xesam:url    │ https://open.spotify.com/track/51748BvzeeMs4PIdPuyZmv │
      ╰──────────────┴───────────────────────────────────────────────────────╯

## `dbus get-all`

    Get all D-Bus property for the given objects

    Search terms: dbus

    Usage:
      > dbus get-all {flags} <object> <interface> 

    Flags:
      -h, --help - Display the help message for this command
      --session - Send to the session message bus (default)
      --system - Send to the system message bus
      --started - Send to the bus that started this process, if applicable
      --bus <String> - Send to the bus server at the given address
      --peer <String> - Send to a non-bus D-Bus server at the given address. Will not call the Hello method on initialization.
      --timeout <Duration> - How long to wait for a response
      --dest (required parameter) <String> - The name of the connection to read the property from

    Parameters:
      object <string>: The path to the object to read the property from
      interface <string>: The name of the interface the property belongs to

    Examples:
      Get the current player state of Spotify
      > dbus get-all --dest=org.mpris.MediaPlayer2.spotify /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player
      ╭────────────────┬────────╮
      │ CanPlay        │ true   │
      │ Volume         │ 0.43   │
      │ PlaybackStatus │ Paused │
      ╰────────────────┴────────╯

## `dbus set`

    Get all D-Bus property for the given objects

    Search terms: dbus

    Usage:
      > dbus set {flags} <object> <interface> <property> <value> 

    Flags:
      -h, --help - Display the help message for this command
      --session - Send to the session message bus (default)
      --system - Send to the system message bus
      --started - Send to the bus that started this process, if applicable
      --bus <String> - Send to the bus server at the given address
      --peer <String> - Send to a non-bus D-Bus server at the given address. Will not call the Hello method on initialization.
      --timeout <Duration> - How long to wait for a response
      --signature <String> - Signature of the value to set, in D-Bus format.
        If not provided, it will be determined from introspection.
        If --no-introspect is specified and this is not provided, it will be guessed (poorly)
      --dest (required parameter) <String> - The name of the connection to write the property on

    Parameters:
      object <string>: The path to the object to write the property on
      interface <string>: The name of the interface the property belongs to
      property <string>: The name of the property to write
      value <any>: The value to write to the property

    Examples:
      Set the volume of Spotify to 50%
      > dbus set --dest=org.mpris.MediaPlayer2.spotify /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Volume 0.5

## `dbus list`

    List all available connection names on the bus

    These can be used as arguments for --dest on any of the other commands.

    Search terms: dbus

    Usage:
      > dbus list {flags} (pattern) 

    Flags:
      -h, --help - Display the help message for this command
      --session - Send to the session message bus (default)
      --system - Send to the system message bus
      --started - Send to the bus that started this process, if applicable
      --bus <String> - Send to the bus server at the given address
      --peer <String> - Send to a non-bus D-Bus server at the given address. Will not call the Hello method on initialization.
      --timeout <Duration> - How long to wait for a response

    Parameters:
      pattern <string>: An optional glob-like pattern to filter the result by (optional)

    Examples:
      List all names available on the bus
      > dbus list

      List top-level freedesktop.org names on the bus (e.g. matches `org.freedesktop.PowerManagement`, but not `org.freedesktop.Management.Inhibit`)
      > dbus list org.freedesktop.*
      ╭───┬───────────────────────────────╮
      │ 0 │ org.freedesktop.DBus          │
      │ 1 │ org.freedesktop.Flatpak       │
      │ 2 │ org.freedesktop.Notifications │
      ╰───┴───────────────────────────────╯

      List all MPRIS2 media players on the bus
      > dbus list org.mpris.MediaPlayer2.**
      ╭───┬────────────────────────────────────────────────╮
      │ 0 │ org.mpris.MediaPlayer2.spotify                 │
      │ 1 │ org.mpris.MediaPlayer2.kdeconnect.mpris_000001 │
      ╰───┴────────────────────────────────────────────────╯
