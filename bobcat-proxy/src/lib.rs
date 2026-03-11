#![no_std]

use array_concat::concat_arrays;

use bobcat_interfaces::selectors;

type Address = [u8; 20];

pub const fn make_minimal_proxy(addr: Address) -> [u8; 18 + 16 + 20] {
    // I can't remember where this comes from (TODO), but we use this in
    // 9lives for our share proxies.
    concat_arrays!(
        [
            0x60, 0x2d, 0x5f, 0x81, 0x60, 0x09, 0x5f, 0x39, 0xf3, 0x5f, 0x5f, 0x36, 0x5f, 0x5f,
            0x37, 0x36, 0x5f, 0x73,
        ],
        addr,
        [
            0x5a, 0xf4, 0x3d, 0x5f, 0x5f, 0x3e, 0x60, 0x29, 0x57, 0x3d, 0x5f, 0xfd, 0x5b, 0x3d,
            0x5f, 0xf3,
        ]
    )
}

macro_rules! unpack_arr {
    ($b:expr, $l:expr) => {
        match const_hex::const_decode_to_array::<$l>($b) {
            Ok(v) => v,
            Err(_) => panic!("bad code"),
        }
    };
}

pub const SIZE_EIP1967: usize = 1 + 20 + 76;

/// Make a EIP1967 proxy that reads from the standard storage slot.
pub const fn make_eip1967_proxy(logic: Address) -> [u8; SIZE_EIP1967] {
    // Created from eip1967.huff .
    concat_arrays!(
      [0x73],
      logic,
      unpack_arr!(
          b"602060305f395f5155603a8060273d393df3365f5f375f5f365f7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc545af45f3d5f5f3e3d9161003857fd5bf3",
          76
      )
    )
}

/// Make beacon proxy that calls "implementation()(address)" on a
/// hardcoded beacon address to get the logic address to delegate to.
pub const fn make_beacon_proxy(beacon: Address) -> [u8; 20 + 110] {
    // Created from beacon-proxy.huff .
    concat_arrays!(
        unpack_arr!(
            b"60408060093d393df3365f5f375f5f365f6020635c60da1b815280806004603c73",
            33
        ),
        beacon,
        unpack_arr!(b"5afa50515af45f3d5f5f3e3d9161003e57fd5bf3", 20)
    )
}

pub const SIZE_UPGRADEABLE_BEACON_PROXY: usize = 1 + 20 + 123;

/// Make a beacon proxy that loads from the beacon slot to get the
/// "implementation()(address)" to get the logic address to delegate to,
/// with the value initially set during the constructor.
pub const fn make_upgradeable_beacon_proxy(beacon: Address) -> [u8; SIZE_UPGRADEABLE_BEACON_PROXY] {
    // Created from beacon-proxy.huff .
    concat_arrays!(
        [0x73],
        beacon,
        unpack_arr!(
            b"7fa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d5055604d8060403d393df3365f5f375f5f365f6020635c60da1b815280806004603c7fa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50545afa50515af45f3d5f5f3e3d9161004b57fd5bf3",
            123
        )
    )
}

pub const SIZE_MULTI3_PROXY: usize = 46 + 20 + 7 + 20 + 7 + 20 + 6 + 20 + 21;

pub const fn make_multi3_proxy(
    one: Address,
    two: Address,
    three: Address,
    all: Address,
) -> [u8; SIZE_MULTI3_PROXY] {
    // Created from multi3-proxy.huff .
    concat_arrays!(
        unpack_arr!(
            b"609e8060093d393df3365f5f375f5f365f6002355f1a8060011461003d5780600214610058576003146100735773",
            46
        ),
        all,
        unpack_arr!(b"61008d565b5073", 7),
        one,
        unpack_arr!(b"61008d565b5073", 7),
        two,
        unpack_arr!(b"61008d565b73", 6),
        three,
        unpack_arr!(b"61008d565b5af45f3d5f5f3e3d9161009c57fd5bf3", 21)
    )
}

pub const SIZE_BEACON_SEL_PROXY: usize = 24 + 4 + 12 + 20 + 30;

/// Create a proxy that calls the selector on the beacon given
/// to figure out where to delegatecall its calldata to.
pub const fn make_beacon_sel_proxy_sel(
    sel: [u8; 4],
    beacon: Address,
) -> [u8; SIZE_BEACON_SEL_PROXY] {
    // Created from sel-beacon-proxy.huff .
    concat_arrays!(
        unpack_arr!(b"60518060093d393df3602460046020368282355f5f368663", 24),
        sel,
        unpack_arr!(b"5f52855f8937808a80601c73", 12),
        beacon,
        unpack_arr!(
            b"5afa1561004f578951368a8c375af43d5f5f3e5f3d911561004f57f35bfd",
            30
        )
    )
}

selectors! {
    // Used by the beacon proxy to implement a fallback.
    SEL_IMPLEMENTATION = b"implementation(bytes4)",
    // Used by the metamorphic proxy to support a migrate feature.
    SEL_MIGRATE = b"migrate()"
}

/// Create a proxy that calls "implementation(bytes4)" on the beacon
/// address given with the selector in the calldata to this proxy. Use the
/// returned address as the target of a delegatecall.
pub const fn make_beacon_sel_proxy(beacon: Address) -> [u8; SIZE_BEACON_SEL_PROXY] {
    make_beacon_sel_proxy_sel(SEL_IMPLEMENTATION, beacon)
}

pub const SIZE_METAMORPHIC_ON_FN: usize = 34 + 20 + 61;

/// Make a metamorphic with an opt-in migrate function proxy.
pub const fn make_metamorphic_proxy(deployer: Address) -> [u8; SIZE_METAMORPHIC_ON_FN] {
    // Created from metamorphic-on-fn.huff .
    concat_arrays!(
        unpack_arr!(b"606a8060093d393df3365f5f375f3560e01c638fd3ab801461001757610031565b73", 34),
        deployer,
        unpack_arr!(
            b"610054565b7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc545b5f365f5f935af43d5f5f3e3d5f8261006857fd5bf3",
            61
        )
    )
}

pub const SIZE_METAMORPHIC_ON_EVMFN_BEACON: usize = 35 + 20 + 69;

/// Make a metamorphic with an opt-in migrate function proxy that calls
/// to the beacon for the implementation address.
pub const fn make_metamorphic_evmfn_beacon_proxy(
    deployer: Address,
) -> [u8; SIZE_METAMORPHIC_ON_EVMFN_BEACON] {
    // Created from metamorphic-on-fn-beacon.huff .
    concat_arrays!(
        unpack_arr!(b"60738060093d393df35f3560e01c638fd3ab801461001357610036565b60205f5f5f73", 35),
        deployer,
        unpack_arr!(
            b"5afa5f51610059565b7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc545b365f5f375f365f5f935af43d5f5f3e3d5f8261007157fd5bf3",
            69
        )
    )
}

pub const SIZE_METAMORPHIC_ON_BORSH_BEACON: usize = 30 + 20 + 69;

/// Make a metamorphic with an opt-in migrate path that works using the
/// first byte of the data that's being provided, assuming the first
/// argument is the Upgrade path.
pub const fn make_metamorphic_borsh_beacon_proxy(
    deployer: Address,
) -> [u8; SIZE_METAMORPHIC_ON_BORSH_BEACON] {
    // Created from metamorphic-on-borsh-beacon.huff .
    concat_arrays!(
        unpack_arr!(b"606e8060093d393df35f3560f81c1561000e57610031565b60205f5f5f73", 30),
        deployer,
        unpack_arr!(
            b"5afa5f51610054565b7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc545b365f5f375f365f5f935af43d5f5f3e3d5f8261006c57fd5bf3",
            69
        )
    )
}
