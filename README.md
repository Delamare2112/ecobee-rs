# Ecobee for Rust!  
This could be used as a starting point for a more complete crate.  Maybe one day I will take the time and finish it!  

## What you get with this
The crate has everything one would need to check the status of most ecobee products and update the thermostat settings.  For example,
I use this to monitor if a door or window has been opened and enable or disable the thermostat accordingly.  Take a look at the [auto_door.rs](https://gitlab.com/Delamare/ecobee/-/blob/master/examples/auto_door.rs) example to see how I do that!

This wrapper around the API is very light and still requires that the user have a decent understanding of the Ecobee API.

## Future improvements
I remember having ideas on how to restructure the lib.rs source to be more easily extendable and maintainable.  However, that was months ago.  And my whole use case mentioned above works really well for me right now so I have had little drive to rewrite something that is currently working for me.
