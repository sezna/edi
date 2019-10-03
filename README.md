Nice to haves that are not yet implemented:
* Output back into EDI format with proper padding
* Support for multiple ISA's or GS's
* Loops are simply not supported. They require a licensed guide to identify.


In the interest of providing a highly usable interface, there are individual specialized structs for certain segment types like the ISA segment (`InterchangeControlHeader`) and the GS segment (`FunctionalGroupHeader`). These are ubiquitous and contain important metadata, so they are lifted into their own struct. Outside of these, other segments are organized into a discriminated unionof type `Segment`. 
