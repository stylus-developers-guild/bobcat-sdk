
# 0.10.0

The first of many changelog updates since our change frequency has
increased!

In this release, we (the Superposition team):

1. Pulled `bobcat-maths-zone` and `tickmath-local`/the Tick Markov Geist
precompiles. W the packages may be good code (in that they achieve
the goal that was set out for them), they've simply not seen any use
internally beyond simple one-offs. We didn't feel it was appropriate to
continue including it. Both packages are pretty niche and fill a very
specific need and are better suited as external packages we feel.

2. Added a derive proc_macro, the `EvmCd*` family. It makes it possible
to derive with `EvmCdSerialise` and `EvmCdDeserialise` to get encoding
to and from EVM calldata. It makes it possible to have a Borsh-style
calldata decoding that know well. It can be used with `bobcat-cd-derive`.

3. Arbitrum precompiles that would unnecssarily return Option are now
returning just the word/whatever argument they returned. The debug_
functions will continue to return Option, because debug is not always
available.

4. Several decoding functions for strings/arrays were finally added to `bobcat-cd`. `*_call_bool_opt` was updated to return `Some(the return result)` to improve its ergonomics. Also added `*_call_word_opt` to return a word with Option.
