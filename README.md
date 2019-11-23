# POS58 USB
Little library to allow writing to a POS58 printer over USB through the `io::Write` trait. This is so that escposify can be usefully interfaced with it, not by my design.

WIP; Right now it is full of bad practice (I.E. Translating errors into io::Error).
