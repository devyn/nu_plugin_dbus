# nu_plugin_dbus

Install with Cargo: `cargo install --locked nu_plugin_dbus`

    Commands for interacting with D-Bus

    Search terms: dbus

    Usage:
      > dbus 

    Subcommands:
      dbus call - Call a method and get its response
      dbus get - Get a D-Bus property
      dbus get-all - Get all D-Bus property for the given objects
      dbus set - Get all D-Bus property for the given objects

    Flags:
      -h, --help - Display the help message for this command

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

