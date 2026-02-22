
# bobcat-precompiles

Calling features for
[https://github.com/fluidity-money/superposition-precompiles](Superposition), Arbitrum, [https://code.markovgeist.org/precompiles](Markov Geist), and EThereum.

Note on `tickmath-local`: this feature will enable the local tickmath calculation instead
of using the precompile contract. This is generally not desirable, and should only be used
in the testing context. The reason is that it's not necessary to cast to and from our sdk
primitive type (U) and the stylus-sdk preferred type, U256. Users making use of stylus-sdk
should prefer the limbs form, or the version that converts directly to u160.
