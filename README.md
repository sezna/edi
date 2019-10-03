Nice to haves that are not yet implemented:
* Output back into EDI format with proper padding

In the interest of providing a highly usable interface, there are individual specialized structs for certain segment types like the ISA segment (`InterchangeControlHeader`) and the GS segment (`FunctionalGroupHeader`). These are ubiquitous and contain important metadata, so they are lifted into their own struct. Outside of these, other segments are organized into a discriminated unionof type `Segment`. 
