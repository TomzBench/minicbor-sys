ATX-CDDL
========

- A subset of CDDL with constrained types.

- Code is generated in Rust with C bindings

- Server side uses Rust code, device side uses C bindings. Data types can then 
  be unified from a "single source of truth". (The CDDL definitions)

Limitations
-----------

- No inline struct definitions (IE: `thing = foo: { x: u8, y: u8 }`)

- Arrays must be fixed length (IE: `users = [ 4*4 user ]`)

- No "Choices" (AKA unions or enums). (IE: `method: "DHCP" / "STATIC"`)

- No "Optional". (IE: `? version = tstr .size 32`)

- All strings or byte arrays must be sized (IE: `serial: tstr .size 48`)

Future plans
------------

- "Choices" and "Optional" features would like to add, however, "nesting" and 
  "unconstrained primitive types" is counter productive to code gen for the target 
  devices. 

- Nesting inline map or group definitions leaves ambiguity about how to generate 
  code for the nested type, and unconstrained primitive types leaves ambiguity about
  how much space to reserve for unconstrained member fields because we do not allow
  heap allocations on target device.

TODO

- Add prefix and correct cases when in bindings

- distinguish uint8_t vs char
