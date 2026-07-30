
# bobcat-create

`bobcat-create` is a collection of high and low level functions for contract creation.
Several functions are provided that do things like set up various types of proxies, and
one that generates contract code that set slots during constructor time. These functions
are used to avoid invoking constructor code on the implementation of the proxies to save
gas.
