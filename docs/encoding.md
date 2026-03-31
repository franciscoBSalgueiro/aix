# Binary encoding of chess games

Aix makes use of a binary encoding for the moves in a chess game (the [movedata column](columns.md) in the Aix-compatible Lichess database). See [the blog post](https://thomasd.be/2026/02/01/aix-storing-querying-chess-games.html#space-efficient-storage) for details.

There are three possible compression levels for the binary encoding: Low, Medium, and High. A lower compression level takes up more disk space, but the decoding speed is higher. The [`recompress` function](functions.md#recompress) can transform encoded games between different compression levels.

This fork supports decoding from a custom initial position only for Low compression. SQL decode functions provide overloads with an `initial_fen` argument; for Medium and High compressed movedata this argument is ignored.
