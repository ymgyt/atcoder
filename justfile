alias t := test

[linux]
test:
    @wl-paste
    @wl-paste | cargo run --quiet

[macos]
test:
    @pbpaste
    @pbpaste | cargo run --quiet
