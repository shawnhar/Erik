# Erik - a calculator

This program is the result of a Week-Of-Learning project for me to learn the Rust 
programming language. I followed the same idea as when I first learned C# by writing a 
calculator program called [Eric](https://shawnhargreaves.com/eric). This one has similar 
features but is implemented in a different language.

My Rust calculator is called Erik, because if the original C# version was named Eric, a 
rusty version of that can surely be nothing other than Erik the Red...

## Features:

    > 2+3
    5
    
    > sqrt(9)+5*2
    13
    
    > f(x,y) = x^(y*2)
    
    > f(3, 4)
    6561
    
    > factorial(n) = n>1 ? n*factorial(n-1) : 1
    
    > factorial(2)
    2
    
    > factorial(3)
    6
    
    > factorial(4)
    24
    
    > base 2 10 13 16
    Using base 2 10 13 16
    
    > 1234
    100_1101_0010  1234  73c  0x4d2

"help" to show all the available operators, functions, and commands.

"q" to quit.


## Observations on Rust:

On the first day, I hated it. Progress was glacially slow, and the language fought me 
every step of the way. There is a steep learning curve to Rust, more so than most other 
languages I have learned. It made me angry and frustrated.

On the second day, some things started to click. I realized that trying to write C++ or 
C# with a Rust syntax was paddling against the current, so I needed to rethink some 
fundamentals. Progress improved.

After one week I am still very much a Rust newbie, but I think well informed enough that 
my opinions are starting to have some merit. And I like it! With a few caveats, but I 
like it.

Pros of Rust:
- Encourages you to do things right. The compiler really does help you catch many common errors.
- The lifespan management system is pretty awesome. Compared to C# where I was just allocating and copying like crazy, or C++ where I'd be partying on pointers to other people's data and hoping those remained valid, Rust allowed me to build a tokenizer that operates on borrowed slices of strings with zero copies, plus a parser that copies the few strings it needs to keep in order to provide self-sufficient output, with robust compiler validation that I got all the transfers of ownership and cloning right when data moves from tokenizer to parser. Really awesome stuff.
- Error handling is fantastic. Unobtrusive to the flow of algorithms, yet robust. This is by far the most clarity a language has ever helped me with around which things can fail and what needs to be checked where.
- The dev environment is solid. Great built-in unit test framework, and it's super easy to consume 3rd party crates (of which many useful ones are available).
- Pragmatic mix of functional, imperative and other programming styles. I appreciate that it doesn't try to force you down any one path.
- Pattern matching struck me as kinda pointless syntax sugar at first, but it grew on me. Can be an elegant way of expressing complex logic.
- From what little I've seen, macros are crazy cool and powerful, but I still need to learn more here.

Cons:
- The learning curve is steep!
- Rust tries so hard to get you to write things correctly the first time, it's slower than other languages for prototyping and quickly hacking stuff together.
- Sometimes its focus on correctness gets in the way of practicality. Really, you aren't going to let me assign a constant 0 to a floating point value unless I write 0.0? And you're not going to let me feed floats into the min/max operators because when considering NaN those technically don't have a strict ordering? Come on, this is pedantry, not helpfulness.
- The standard library is oddly inconsistent. Operations available on arrays, strings, slices, vec, and iterators are significantly different, so you have to pay attention to which of these you are dealing with and frequently insert iter() or collect() calls to convert from one to the other depending on what you want to do next. This is annoying, and a notable contrast vs. C# where _everything_ implements IEnumerable in a super consistent way.
- Trivial nit: I kept getting tripped up by => vs. ->! Do those really need to be different?
- Another nit: the distinction between statements (terminated by ;) and expressions (no terminator) seems needlessly subtle.
