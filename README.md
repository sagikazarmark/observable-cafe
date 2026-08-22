# The Observable Café

![The Observable Café](resources/screenshot.png)

An interactive example for exploring metrics, either on its own or embedded in
a course. A café owner sells coffee and keeps an eye on two thermometers, and
every so often writes down what things look like.

There is one page, and it is the café. What the café shows is decided when it
starts and never changes afterwards, so a course that wants to introduce one
idea at a time runs a café that shows only that idea. Nothing about it is
chosen in the browser: a page has nothing to ask for and no way in.

## What it can show

| Feature                  | What it adds                                                        |
| ------------------------ | ------------------------------------------------------------------- |
| `header`                 | The sign the café opens with: its mark and its name, left of the clock. |
| `notebook`               | The notebook itself, and with it everything kept in it.             |
| `observations`           | The entries: the café written down every so often, and the button that asks for one now. |
| `automatic-observations` | The clock writing those entries as their interval comes due, and the box that paces it. |
| `sales`                  | Every sale, written up on the roll as it happens.                   |
| `labels`                 | Counts broken down by which drink it was.                           |
| `types`                  | The same entries sorted by metric, where a number is named as a counter or a gauge. |

A café nobody has configured shows all of them. That is the whole café, and it
is what `observable-cafe` on its own serves.

Two rules join them up, so that turning something off does not have to be
followed through by hand:

**A feature kept in the notebook is not shown when the notebook is not.**
Turning off the notebook is the short way of asking for a café that writes
nothing down where anybody can see it, and it goes on meaning that when another
feature is kept in the notebook later. Observations work the same way: turning
them off stops the timer too, whether or not the timer was named.

None of this reaches `/metrics`. That is built from the café rather than from
the notebook, and it reads no feature at all: a café showing no notebook still
sells coffee, still moves its thermometers, and still publishes every series.

**A notebook with nothing in it is not drawn.** Turn off observations, sales
and types and it does not appear as an empty frame; the café takes the page to
itself. That is what makes a café showing nothing but coffee and thermometers
worth serving.

Neither rule reaches the sign, which is over the door rather than in the
notebook: a café keeping no record at all still says whose café it is. It is
answered on its own, and `--disable header` is what a page that has already
named the café asks for, since saying so twice costs height the record is
worth more of.

## Presets

A preset is a named set of features to start from, and it starts from nothing:
it shows what it names and no more. That is deliberate. A feature added to the
café later joins the café that was asked for by default and none of the
presets, so an example built against `samples` goes on showing exactly what it
showed when it was written.

| Preset    | What it shows                                                     |
| --------- | ----------------------------------------------------------------- |
| `samples` | The notebook, filled on a timer. A record is made of samples, and whatever happens between them is not kept. |
| `labels`  | The same, with counts broken down by drink.                       |
| `types`   | The same again, with the metrics view: a counter and a gauge are different kinds of number. |

These are the three stages the café used to be a ladder of, kept because a
course page built against one of them should go on working. None of them names
the sign, which arrived after all three; `--enable header` puts it back.

`--enable` and `--disable` then have the last word, so an example can start
from a preset and still differ from it in one place:

```shell
observable-cafe --preset samples --enable sales
observable-cafe --preset types --disable automatic-observations
observable-cafe --disable notebook
observable-cafe --disable header       # embedded in a page that already names it
```

Naming a feature in both is refused rather than resolved one way or the other:
nobody means both, so it is a mistake to report rather than a preference to
honour.

## Endpoints

`/metrics` is the scrape endpoint, in the format every scraper reads. There is
one of them and it is always the full exposition, whatever the page is showing,
so it always looks like a real target.

`/version` reports which build is running, as one line of plain text. Every
page also carries the same string in its head as
`<meta name="version" content="…">`, so a tab that is already open can be
identified without going back to the server.

Everything else is the café. There is one page and every path serves it, so a
link written against the old stage URLs lands on the café rather than on an
error.

## How it is meant to be read

**Selling a coffee changes no number in the notebook.** A toast confirms the
sale and that is all: the counter behind the till moves, but nobody has written
anything down yet. Seven seconds later the owner looks up and one entry
appears, however many coffees were sold in between. That gap is the point.

**The sales are what the notebook is a sample of.** Three sales on the roll and
one entry in the notebook is the whole idea, and it only lands if both can be
counted, which is why they are two tabs rather than one list. The roll is
printed and the notebook is handwritten so that the two are told apart at a
glance. Without `labels` a sale says only that a coffee was sold: the café
knows which drink it was, and this record does not keep the dimension, which is
what a café without labels actually looks like.

**The café runs faster than the world.** One real second is one café minute, so
the clock in the corner sweeps through a working day in about a quarter of an
hour, and the thermometers follow a real daily arc: cool at opening, warmest
mid-afternoon. The temperatures also follow the calendar, so a café opened in
January is a colder day than one opened in July.

**The clock does not start until somebody looks.** A server nobody has visited
sits at 08:00 with an empty notebook, so the first thing a learner sees is the
café opening rather than however many hours it drifted through while the
process happened to be running. Scraping `/metrics` does not start it either: a
scrape is an observer, not a visitor.

**The thermometers move every five to ten seconds, and the notebook catches
only some of it.** A rise and fall that happens between two entries leaves no
trace anywhere, which is the failure mode worth meeting early. The gap between
readings is deliberately not a fixed number: weather keeps no schedule, and a
fixed gap against a fixed interval would lock the two together so that the same
movements fell into the same blind spot every time.

**How often the café is written down is the reader's to choose.** The box in
the bar takes anything from 5 to 30 seconds and starts at 7; anything outside
that snaps back, on the page and again at the café, which does not trust a
number that arrived over the wire. Widening it is the point: the café goes on
moving at exactly the same rate, and the record simply keeps less of it.
Changing it disturbs nothing; entries just start arriving at the new spacing,
so one notebook holds fine-grained history above and coarse below, to be
compared directly. At 30 seconds the notebook misses about four temperature
movements per entry; at 5 it usually misses none.

Because the café clock runs at a minute a second, the interval is legible in
the record itself: 5 seconds gives `08:00, 08:05, 08:10`, and 30 gives `08:00,
08:30, 09:00`. The note under the clock says so, since otherwise a box reading
"7s" and a notebook showing seven-minute gaps look like a contradiction.

**"Observe now" outlives the timer.** Turning off `automatic-observations`
hands the notebook over to whoever is reading rather than closing it, so a
demonstrator can sell three coffees and then write the one entry that records
them. Turning off `observations` closes it: the button goes, the timer goes,
and the endpoint behind the button writes nothing even when something calls it
directly. A café asked to write nothing down writes nothing.

**The charts fit their readings, not the thermometer.** Drawn against
everything the instrument could possibly show, an ordinary afternoon occupies
under a third of the height and a degree is worth about two pixels, so the café
looks becalmed when it is not. Each chart therefore fits the window it is
showing, never narrower than six degrees, which keeps a genuinely quiet stretch
looking quiet instead of turning noise into a mountain range. Because the scale
moves, each chart says underneath what it is fitted to. The thermometers keep
their fixed printed range, as a physical instrument would. A café with `types`
but no `observations` draws no chart at all: the reading above it is the whole
of what it knows, and an empty frame would suggest a history that was never
kept.

**A page left open runs past midnight**, so the owner rules off and heads the
new day, and the date beside the title follows the newest entry rather than
opening day. Nights are cold, which the calendar already handles.

This has nothing to do with the interval: the café clock advances a minute per
second of being watched, so midnight always arrives sixteen minutes after the
page is opened. The interval only decides how many entries span that.

**`/metrics` is a separate page, not a panel.** It is plain text, byte-identical
to what `curl` sees, and it does not update on its own. Reloading it *is* a
scrape: nothing pushed those numbers anywhere; something came and asked. It
reports the café as it stands the instant it is asked and stores nothing, so its
numbers run ahead of the notebook's.

## Development

```shell
dx serve
```

Then open <http://localhost:8080>. The scrape endpoint is at
<http://localhost:8080/metrics>, and <http://localhost:8080/version> says which
build is answering.

```shell
dx serve --args="--preset samples"
dx serve --args="--preset labels --enable sales"
dx serve --args="--disable automatic-observations"
dx serve --args="--disable notebook"   # coffee and thermometers only
dx serve --args="--disable header"     # embedded in a page that already names it
```

`--enable` and `--disable` take several features at once, separated by commas
or repeated.

All of it can also be set with environment variables. A variable holds one
string, so lists are written the way the command line writes them rather than
as `[a,b]`:

```shell
OBSERVABLE_CAFE_PRESET=samples dx serve
OBSERVABLE_CAFE_ENABLE=sales,types dx serve
OBSERVABLE_CAFE_DISABLE=labels dx serve
```

Or in a settings file, which saves repeating them for a café that is run the
same way every time. An `observable-cafe.toml` in the working directory is read
if there is one, and `--config` or `OBSERVABLE_CAFE_CONFIG` names one somewhere
else. Only the working directory is searched, so there is one place to look
when a setting turns out not to be what was expected.

```toml
# observable-cafe.toml
preset = "samples"
enable = ["sales"]
```

The three sources layer, each overruling the one before it: the file, then the
environment, then the command line. A list is taken from one layer whole rather
than added to across layers, so overruling one means giving the list that
replaces it. A file that was asked for by name and is not there is an error,
since the café would otherwise run on defaults that look nothing like what was
meant; the default file is not asked for, so a café without one is simply a
café configured some other way, or not at all.

`dx serve --args="--help"` lists the options and every feature name, and
`--args="--version"` says which build would answer without having to ask
`/version` for it. Anything else — an unknown argument, a feature that is not
one, an `OBSERVABLE_CAFE_` variable that names no setting, a key the file does
not have — is refused rather than ignored, so a misspelling is not mistaken for
the default.

```shell
cargo test --no-default-features --features server
dx build --release   # bundles into target/dx/observable-cafe/release/web/public
cargo clippy --no-default-features --features server
cargo clippy --target wasm32-unknown-unknown --no-default-features --features web
cargo fmt
```

The `web` and `server` features pick which half of the app is being built; `dx`
sets them for you. The tests cover what the café is told to show and what it
makes of it, which is where the rules above either hold or do not.

## Layout

| Path                       | Contents                                            |
| -------------------------- | --------------------------------------------------- |
| `src/feature.rs`           | What the café can show, and what a preset adds up to |
| `src/config.rs`            | Where the settings come from, and how they layer    |
| `src/app.rs`               | Root component: polling, page layout                |
| `src/components/`          | Header, menu, thermometers, notebook, observations, sales, cards |
| `src/clock.rs`             | The café's own clock, which everything reads time from |
| `src/menu.rs`              | What the café sells, and the label values it publishes |
| `src/state.rs`             | What the server observes and hands to the browser   |
| `src/season.rs`            | What the calendar says a thermometer should read    |
| `src/api.rs`               | Server functions the browser calls                  |
| `src/server.rs`            | The café: in-memory state and the axum router       |
| `src/server/simulation.rs` | Readings that move on their own                     |
| `src/server/metrics.rs`    | The `/metrics` scrape endpoint                      |
| `src/server/version.rs`    | The `/version` endpoint                             |
| `src/server/rng.rs`        | Seeded xorshift used to simulate readings           |
| `assets/main.css`          | Styles                                              |

## Metrics

```
cafe_coffees_sold_total{drink="…"}   counter, one series per drink sold
cafe_inside_temperature_celsius      gauge
cafe_outside_temperature_celsius     gauge
```

There is one `/metrics`, and it does not vary with what the page shows: always
the full exposition, so it always looks like a real target. A café serving no
labels on screen still publishes the per-drink series here, which is a fair
picture of the usual situation: the instrument records more than the page in
front of you shows.

A drink nobody has ordered has no series at all, rather than a series reading
zero; a series begins when something is first observed under it, which is why
graphs come out with gaps in them.

There is deliberately no unlabelled `cafe_coffees_sold_total` alongside the
per-drink series: a total published next to its own breakdown would be counted
twice by anything summing the lot. The notebook still writes a total, because a
person keeping notes wants the headline; adding the series up is the reader's
job.

## Known gap

The café is a single global (`src/server.rs`), so every visitor shares one
counter, one notebook and one roll of sales. That is what lets a cookie-less
`curl` see the same numbers as the page, but it also means another learner's
purchases arrive as unexplained coffees, which undercuts the sampling lesson,
where it matters that you can check that you clicked three times and one entry
appeared. Scoping state per visitor would fix that and break the `curl`; it has
not been decided yet.

The observation interval is shared the same way, and it is the one thing a page
still tells the café. The last poll wins, so two readers who have chosen
different intervals will pull the cadence back and forth between them. Nothing
is destroyed by it, but concurrent use needs the state-scope question settled
first. A single reader, or several reading at the same interval, is fine.

Because the café is never rebuilt for a new arrival, only the first visitor
finds it at opening time; whoever arrives an hour later finds an afternoon
already in progress and a notebook part-full. "Reset demo" is the answer, and it
was already the answer for a café serving a single stage, which never rebuilt
itself either.

Relatedly, the global `Mutex` and the long-lived background task rule out plain
Cloudflare Workers. Publishing there would need Durable Objects, or a host that
can keep a process running.
