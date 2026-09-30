
"""""""""""""""
Getting started
"""""""""""""""

Bobcat-sdk can be started with quickly by either using a sh-based script that can be
pasted (`bobcat-up`), simply adding to an existing Rust project, or a scaffolding script.

Installation
------------

We can get started with Bobcat-sdk quickly using a setup script piped. This script creates a directory
and adds Bobcat-sdk. It creates the entrypoint function and a basic entrypoint type.

.. code-block::

	curl https://raw.githubusercontent.com/stylus-developers-guild/bobcat-sdk/refs/heads/trunk/bobcat-quickstart.sh | sh

Cargo-add
"""""""""

.. code-block::

	b % cargo add bobcat-sdk
	    Updating crates.io index
	      Adding bobcat-sdk v0.11.2 to dependencies
	      ...


Bobcat-new
""""""""""

`bobcat-new.sh`: This script quickly scaffolds a Stylus bobcat-sdk project, including a
Git repo and some helpers to get started quickly with testing.

.. code-block::

	b % git clone https://github.com/stylus-developers-guild/bobcat-sdk
	b % cd bobcat-sdk
	b % cp bobcat-new.sh $HOME/.bin
	b % cd $HOME/Downloads && bobcat-new.sh example-repo


Hello world
-----------

This is a simple contract that writes "Hello, world!" back to the caller when invoked:

.. code-block::

	// src/main.rs

	use bobcat_sdk::prelude::*;

	#[unsafe(no_mangle)]
	fn user_entrypoint(len: usize) -> usize {
	    write_str("Hello, world!");
	}

This smart contract simply writes "Hello, world!" as a string when it's invoked.
