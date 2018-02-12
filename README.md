# rx3wm

Fork of [`xr3wm`](https://github.com/tsurai/xr3wm).

i3 and xmonad inspiered tiling window manager written in Rust.

## Why fork?

The original `xr3wm` crashed repeatedly on my system. The version I cloned from
GitHub did not even build. I want a working WM written in Rust that I can hack
on in my free time. [`wtftw`](https://github.com/Kintaro/wtftw) doesn't fit the
model I'm interested in (basically, an i3 clone in Rust) so forking `xr3wm` is
the most obvious choice.

Besides that, I'd rather fork an existing WM that is _almost_ there than write
one from scratch. I'm a busy guy and don't want to waste time learning how
window managers work when I can get a head-start using open-source code.

## Running

This software is super alpha, so log your debug output:

```
$ startx &>> ~/xr3wm.log
```

## ToDo

- [ ] never crash
- [ ] i3 style manual splitting
- [ ] improve config usability
- [ ] user documentation
- [x] load config as dynamic library

## Credits

Please have a look at [Kintaros](https://github.com/Kintaro) wm [wtftw](https://github.com/Kintaro/wtftw). He helped me a lot with my implementation and stupid questions.
