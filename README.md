# tlsign 
A small command line interface to sign POST requests for Payouts/Paydirect API

```
USAGE:
    tlsign --body <body> --key <key> --kid <kid>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --body <body>    The payload you want to sign
        --key <key>      The filename of the Elliptic Curve private key used to sign, in PEM format
        --kid <kid>      The certificate id associated to the public certificate you uploaded in
                         TrueLayer's Console. The certificate id can be retrieved in the Payouts
                         Setting section. It will be used as the `kid` header in the JWS
```

## Install
`cargo install --git https://github.com/tl-alex-butler/tlsign`

_**Or**_

`cargo install --git ssh://git@github.com/tl-alex-butler/tlsign`

_**Or**_

Checkout the project. Build a release binary and copy to a directory _(default ~/bin)_ with:

`./deploy [TARGET_DIR]`
