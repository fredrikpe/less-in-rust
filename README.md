# A less clone in Rust. work in progress

## Why?
For fun and to try rust.

## Random Thoughts
Handling utf-8 is tough! A 1 byte char does not necessarily represent one "real" character. Several chars are sometimes combined to make single on-screen charaters. This means it is not clear when you might have reached the terminal width, and should break the line. Fortunately there's a crate for that [unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation). This crate let's you iterate over "graphemes" which are the actual characters that take up one square on the screen.

The same issue arises when seeking to an arbitrary position in a file and you want that position to be at a valid point between unicode chars. Here using the above crate wasn't possible (you would need to parse the whole file which is too slow), so I used some of the rust standard libraries internal utf-8 validation functions to step forwards/backwards until a small buffer forwards is valid.

When jumping to an arbirtary position in a large file where do you start printing? Preferably at a nearby linebreak, so you can search for that, but there might not even be any! Less doesn't handle single line files very well, you can't do percentage jumps at all, and other jumps (page, half page, to end) requires parsing to that point. I haven't really looked into why. This version lets you percentage jump.

I was interested to see if using the grep crate from [ripgrep](https://github.com/BurntSushi/ripgrep) would produce faster searching than in less, and it does seems slightly faster. I haven't done any benchmarking. Also the way this program searches is a bit trivial. It currently always searches the whole file, as opposed to less which I think only searches until it has found enough matches. Searching should ideally be done in a different thread though.
