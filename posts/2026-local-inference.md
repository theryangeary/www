The new Gemma 4 models were released last month. Some of them are pretty small
(just a few GB) and supposedly are pushing the boundaries of what small models
are capable of. It got me thinking about whether it's feasible that there's a
future where running a smaller model locally on your own machine could be the
future of agentic programming. I know the models obviously won't be as smart as
frontier models are today, but I wanted to try out these smaller models and see
what they were capable of.

## First Model: unsloth/gemma-4-E4B

I have a 2021 M1 Macbook Pro with 16 GB of RAM, which is really not a lot to
work with for local inference purposes.

I installed llama.cpp and tried out
[unsloth/gemma-4-E4B-it-GGUF:UD-Q4_K_XL](https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF)

I also installed the [pi agent](https://pi.dev/), which is a newer-to-the-scene
alternative to claude code or opencode.

Using the pi agent, this model was _kind of_ capable of going back and forth
conversationally, but it became pretty clear pretty quickly that it wasn't going
to be particularly useful for doing any sort of agentic coding tasks. It wasn't
really smart enough to follow any skills that I specifically instructed it to
such as
[/grill-me](https://github.com/mattpocock/skills/tree/main/skills/productivity/grill-me).

It is probably smart enough that if you tell it to run one particular command it
will run that one command, but I wouldn't have a lot of confidence it's going to
be able to do much else reliably.

## Second Model: Variants of Qwen

I then tried out a few different Qwen3 models of similar sizes, but found that
there was something wrong with how they handled tool calls with pi. They would
just print out a code block with the intended tool call to the chat instead of
actually triggering a tool call. For my little experiment, I didn't think it was
worth my time to debug that.

So I found a different model based on the same Qwen base,
[Jackrong/Qwopus3.5-9B-Coder](https://huggingface.co/Jackrong/Qwopus3.5-9B-Coder-GGUF).
This was where things started to get interesting. I fed this model the grill-me
task from above along with a prompt to create the classic video game Pong, with
the reasoning that Pong is a well-defined game where the constraints of the game
would already be encoded in the model rather some unique new game, where I would
have to use context to tell the model what the rules of the game.

And this _kind of_ worked... for a bit. I told it to grill me and to create a
plan to implement Pong. It did not grill me, but it did create a plan! At that
point I created a new session and told it to implement the plan. And it tried.
Eventually, it just got stuck in a loop of compile, try to fix, compile again,
try to fix. And it couldn't get out of the loop. And it couldn't fix the compile
errors.

I have to stress that this model was definitely more capable of the Gemma model
or what I toyed with with the Qwen models that couldn't use tools, but I didn't feel like it was a model that was performing well enough to be meaningfully useful.

## Third Model and Upgraded Hardware

At this point, I felt satisfied that I could conclude that in the present day,
with my present laptop, it was not worthwhile to be running these local models
for agentic coding tasks. This is probably not a shocker.

But I did want to see how far we could get if we upped the specs a little bit. I
got my hands on another M1 MacBook Pro, but this time with 64GB of RAM. So, this
will have similar performance but can use larger models. At this point I decided
to give the model
[unsloth/Qwen3.6-27B-MTP-GGUF](https://huggingface.co/unsloth/Qwen3.6-27B-MTP-GGUF?local-app=llama.cpp)
a try. This has 27B parameters and I'm no LLM expert but 27B parameters is a lot
omre than the 9B from the previous model or the 4B from the Gemma model.

And at this point, it started to be sort of useful! I test this with converting
my personal website (the one you're reading this blog post on) from using a
backend server binary to generating a static site that could be hosted on GitHub
pages[^0]. I told it the site structure should stay the same, but now the binary
should spit out a directory of html files instead of running a long running
server.

It generated a preliminary plan, and then actually started using the /grill-me
skill! This was the point where I felt like we had something that might actually
be worth using.

## Proof of Work

After a short grilling session, there was a plan in place, and the plan looked
good to me, so I started a /new session and just told the model (same Qwen3.6
27B model) to go implement the plan. And it did! I didn't really babysit it or
anything, just walked away, came back a while later, and it was done. And it
legitimately solved the problem correctly. It produced a working set of code
changes such that when I ran `cargo run` it properly generated the `output/` dir
with the appropriate static site resources in it, and it created a working
Github pages release action. All I had to do at that point was `git add . && git
commit`.

The commit is
[here](https://github.com/theryangeary/www/commit/b6abba2c475848d563eee3e661cd620021213655#diff-42cb6807ad74b3e201c5a7ca98b911c5fa08380e942be6e4ac5807f8377f87fc).
I genuinely did not write a line of it.

## In Review

These smaller local models certainly do not have the same reasoning capabilities
(or, perhaps ability to generate tokens that look like reasoning) as the
frontier models, but at some point I expect that to change. I think that there
are a lot of upsides to a sort of democratization of inference, where people can
run useful models on their own hardware, so their data never leaves their own
device or network. It also results in better use of existing hardware (i.e. my
laptop that is sitting here doing nothing while claude code churns in a data
center can become my laptop actually making use of its processor and RAM) and
reduces need for massive data center buildouts.

I've also been feeling lately that frontier models are at or very near the point
of being "good enough" that I don't necessarily care about them getting any
smarter (for the purposes of agentic software engineering at least), although I
wouldn't mind them owning up to when they actually don't know versus confidently
lying to my face. So I personally wouldn't mind if the whole LLM industry
stopped trying to make the models smarter and instead just worked on making them
more efficient in terms of resources needed[^1].

About a year ago I was trying claude code in my free time to see how good it
was, what it was capable of, etc. And I overwhelmingly felt that it was
impressive, but fell pretty flat when I used it at my day job. It just couldn't
keep up with such a large codebase. I absolutely don't feel that way anymore.
Inside of a year, it went from useful only on small personal projects to useful
for all of my work coding tasks.

So presently the small locally runable models aren't quite as useful as claude
code was a year ago for small personal projects, but the models are evolving so
quickly; I expect eventually they will be. At what point do the models become smart and
efficient enough that I can just run them on my commodity hardware rather than
shelling out to someone else hosting it publicly? When will that happen for
small personal projects, and when will it happen for larger projects like I deal
with at $DAYJOB? Probably not tomorrow but I do wonder if it might be inside of
a year from now.

## Footnotes

[^0]: After having built my personal website with HTMX and maud and serving it
with a Rust Axum server, largely motivated by a desire to try out both HTMX and
maud, I've come to the conclusion that I made a little boo-boo, and this website
does not need to have a live server and can in fact just be a static website
hosting on GitHub pages. It is still using Maud for the html generation because
there is little point in changing it at this point, and I like the compile time
guarantees.

[^1]: Just because I think they are "good enough" does NOT mean I think they are
infallible; they are certainly still quite flawed, and require the operator to
know what they are doing to produce the best results.
