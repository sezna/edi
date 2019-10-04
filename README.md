Nice to haves that are not yet implemented:
* Loops are simply not supported. They require a licensed guide to identify.


In the interest of providing a highly usable interface, there are individual specialized structs for certain segment types like the ISA segment (`InterchangeControlHeader`) and the GS segment (`FunctionalGroupHeader`). These are ubiquitous and contain important metadata, so they are lifted into their own struct. Outside of these, other segments are organized into a discriminated unionof type `Segment`. 
 # TODO
  * have a currently-being-worked-on interchange that closes off when an IEA is encountered.  Work with this "pending group" model throughout the ladder.
  * More e2e tests with expected failures as well as other formats
  * benches
  * output back into EDI with proper padding
    