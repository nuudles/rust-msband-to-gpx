# MS Band Run to GPX Converter in Rust

This project provides a simple Rust-based command-line tool to convert JSON files from the [MS Health Dashboard](https://dashboard.microsofthealth.com) to a GPX format that can be consumed by sites such as [Strava](https://strava.com).

## Features

- Converts MS Health Run JSON data to a generic GPX format
- Also includes heartrate data

## Requirements

- Rust 1.6.0?
- Cargo

## Usage

Clone this repo and then execute:

```sh
cargo build
```

To build 

Then execute the `rust-msband-to-gpx` command with the desired JSON file and an optional output filename:

```sh
./rust-msband-to-gpx 2016-01-25.json
./rust-msband-to-gpx 2016-01-26.json MyRun
```

But how do we get these coveted JSON files? Simply login to the [MS Health Dashboard](https://dashboard.microsofthealth.com), then go to this URL:

`https://dashboard.microsofthealth.com/summary/dailyrunsdetail?date=2016-01-25&utcOffsetMinutes=-300`

replacing the date with whichever date you wish to pull and a proper UTC offset. You can then save out the JSON file.

## Caveats

I've only tested this with a couple of my own run files, so it may not work at all for you. If it doesn't, feel free to create a ticket and send me an example JSON file and I can take a look.

## The Story

I got a Microsoft Band fitness tracker a little while ago when it was on sale and use it to track my runs/bike rides. Strava is my main fitness tracking website and MS Health has a Strava syncing plugin, but every so often a run will simply refuse to sync the run. Just recently I had two runs that would refuse to sync to Strava.

Once a sync has failed, there is no official way to manually export the data from the Band, although you could try something like [Unband](http://unband.nachmore.com/). I got tired of having these gaps in my Strava Training Log so I figured there must be a way to get the files. I busted out my Chrome network log on the MS Health dashboard and found the `dailyrunsdetail` JSON files. They looked like they held enough data to create a GPX file from, so I decided to create a quick tool to convert them.

I'm an avid [Swift](https://github.com/apple/swift) developer, and my initial thought was to bang out a quick Swift command-line tool. I have played around with [Rust](https://www.rust-lang.org/) a bit though and have friends who have been touting its praises, so I wanted to see what it would take to create the simple tool in Rust. I banged this out over a few hours to the point where it works, but it's probably pretty terrible Rust code.

I'll use this as an opportunity to clean up my understanding of the Rust language and clean up the code as I go along, so this is mostly a learning repo for me. I may also still create a Swift version of this tool as an interesting comparison between the two languages.

## License

MIT License