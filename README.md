## Observer

Dependencies: gtk4, pkg-config

Works on Linux, MAC and Windows.
Dark theme does not work on MAC (only the app header gets dark)

Note that you might have to `cargo update` before running. You might also need to add some fields
in CreateTarget struct (observer.rs:80) based on the headless_chrome version you use. If the compiler complains about some missing 
fields, just add them with a None value.

### CLI --help
<img width="589" height="944" alt="cli-help" src="https://github.com/user-attachments/assets/8be498cc-9a3b-4fd1-b049-b3fd74ae1a76" />

### GUI mode
<img width="542" height="504" alt="gui-1" src="https://github.com/user-attachments/assets/20da2c4f-a6b5-4615-af6d-2e84dc7609b5" />


#### GUI mode with yaml config file
<img width="545" height="508" alt="gui-2" src="https://github.com/user-attachments/assets/004a1552-f9ad-47d1-a259-d2d399d1d2c8" />


