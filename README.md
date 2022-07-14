# memory.lol

[![Rust build status](https://img.shields.io/github/workflow/status/travisbrown/memory.lol/rust-ci.svg?label=rust)](https://github.com/travisbrown/memory.lol/actions)
[![Coverage status](https://img.shields.io/codecov/c/github/travisbrown/memory.lol/main.svg)](https://codecov.io/github/travisbrown/memory.lol)

## Overview

This project is a tiny web service that provides historical information about social media accounts.

It can currently be used to look up 542 million historical screen names for 443 million Twitter accounts.
Most of this data has been scraped from either the [Twitter Stream Grab][twitter-stream-grab] or the
[Wayback Machine][wayback-machine] (both published by the [Internet Archive](https://archive.org/)).

Coverage should be fairly good (for non-protected accounts) going back to 2011, which is when the Twitter Stream Grab was launched.

Please note that this software is **not** "open source",
but the source is available for use and modification by individuals, non-profit organizations, and worker-owned businesses
(see the [license section](#license) below for details).

## Safety

All information provided by this service has been gathered from public archives, and in most cases it can easily be found through other means
(such as a Twitter search for replies to an account).
The goal of the service is to make it easier for researchers or journalists to identify directions for further investigation,
and more generally to indicate to users that an account may be operating a scam, spreading disinformation, etc.
If you have concerns about safety or privacy, you can contact me (via [Twitter DM](https://twitter.com/travisbrown) or [email](mailto:travisrobertbrown@protonmail.com))
and your request will be handled privately.

## Use cases

Accounts that engage in hate speech, scams, harassment, etc. on social media platforms
sometimes try to obscure their identities by changing their screen names, and they often also have really bad opsec (for example using real names or other identifying information on accounts that they later intend to use anonymously).

Being able to look up historical social media profiles often makes it possible to identify the offline identities of these people (or at least to trace connections between their activities).

Here are a few examples off the top of my head (the first three are examples of the service in action, and the last two show how it can be used to confirm the work of others):

* [**`@OSINT_Ukraine`**](https://memory.lol/tw/OSINT_Ukraine): gained a large following in February 2022; looking up old screen names shows that [it had previously been an NFT scam account](https://twitter.com/travisbrown/status/1496784753705598977).
* [**`@libsoftiktok`**](https://memory.lol/tw/libsoftiktok): a [viral hate account](https://www.washingtonpost.com/technology/2022/04/19/libs-of-tiktok-right-wing-media/) that targets LGBTQ+ people; looking up her screen name in this service is how I found her name (Chaya Raichik) a couple of months ago.
* [**`@_lktk`**](https://memory.lol/tw//_lktk): an abusively transphobic troll named Iratxo Lorca who has been active in the Scala community for years; he was [one of the first people](https://gist.github.com/travisbrown/a704b52d3013471321e5ee6a6b3ff9e6) I identified using this service.
* [**`@Mormonger`**](https://memory.lol/tw/Mormonger): a homophobic Mormon who was [identified as a person named Cole Noorda](https://exposedeznat.noblogs.org/tag/cole-noorda/) last September; this service confirms that he had previously used the screen name `@colenoorda` for his account.
* [**`@_14words_`**](https://memory.lol/tw/_14words_): an account that was [identified as white supremacist Illinois cop Aaron P. Nichols](https://accollective.noblogs.org/post/2022/04/01/magic-dirt-farmer/) earlier this year; this service connects this account to the screen name `@spd584`.

In many cases the information provided by the service won't be enough to identify a person, but may provide hints about where to look next (for example looking up deleted tweets for old screen names with ✨[cancel-culture][cancel-culture]✨ is often a reasonable second step).

## Detailed example

If you visit [`https://memory.lol/tw/libsoftiktok`](https://memory.lol/tw/libsoftiktok) in your browser, you'll see the following data:

```json
{
  "accounts": [
    {
      "id": 1326229737551912960,
      "screen-names": {
        "chayaraichik": null,
        "cuomomustgo": null,
        "houseplantpotus": null,
        "shaya69830552": [
          "2020-11-10"
        ],
        "shaya_ray": [
          "2020-11-27",
          "2020-12-17"
        ],
        "libsoftiktok": [
          "2021-08-18",
          "2022-06-16"
        ]
      }
    }
  ]
}
```

Note that for some screen names we don't currently have information about when they were observed (e.g. the ones with `null` values above).
If an screen name was observed on only one day in our data sets, there will be a single date.
If there are two dates, they indicate the first and last day that the screen name was observed.

These date ranges will not generally represent the entire time that the screen name has been used (they just indicate when the account appears with that screen name in our data sets).

## Other features

The service is very minimal. One of these few things it does support is querying multiple screen names via a comma-separated list (for example: [`https://memory.lol/tw/jr_majewski,MayraFlores2022`](https://memory.lol/tw/jr_majewski,MayraFlores2022)).
It also supports searching for a screen name prefix (currently limited to 100 results; for example: [`https://memory.lol/tw/tradwife*`](https://memory.lol/tw/tradwife*)).

It currently only supports JSON output, but if you want a spreadsheet, for example, you can convert the JSON to CSV using a tool like [gojq][gojq]:

```bash
$ curl -s https://memory.lol/tw/jr_majewski,MayraFlores2022 |
> gojq -r '.[].accounts | .[] | .id as $id | ."screen-names" | keys | [$id] + . | @csv'
89469296,"LaRepublicana86","MayraFlores2022","MayraNohemiF"
726873022603362304,"JRMajewski","jr_majewski"
1533878962455293953,"jr_majewski"
```

Or if you want one screen name per row:

```bash
$ curl -s https://memory.lol/tw/jr_majewski,MayraFlores2022 |
> gojq -r '.[].accounts | .[] | .id as $id | ."screen-names" | keys | .[] | [$id, .] | @csv'
89469296,"LaRepublicana86"
89469296,"MayraFlores2022"
89469296,"MayraNohemiF"
726873022603362304,"JRMajewski"
726873022603362304,"jr_majewski"
1533878962455293953,"jr_majewski"
```

Note that screen name queries are case-insensitive, but the results distinguish case
(which can be useful for archives such as [Archive Today][archive-today], which only provide case-sensitive search).

## Other endpoints

You can also look up an account's history by account ID (e.g. [`https://memory.lol/tw/id/1326229737551912960`](https://memory.lol/tw/id/1326229737551912960) also shows the screen names for Raichik's account).

## Importing data

The application currently supports importing data in two file formats.
The first requires one [Twitter user object][user-object] [per line][ndjson]
(in JSON format with an additional `snapshot` field representing the observation time as an epoch second).
The second is a CSV format with at least three columns (Twitter user ID, screen name, and observation time as epoch second).

## Future

Anything about the web service is subject to change at any time, including its availability.

There are non-public endpoints that I'm likely to open up at some point.
These provide full historical user profiles, information about suspension or deactivation status, etc.

## Terms of service compliance

This web service simply provides an interface to an index for content that is hosted in public archives,
and the project aims to be compliant with the terms of service of all platforms that were accessed
in generating this index.

This repository does not contain data from any social media platform.

## License

This software is published under the [Anti-Capitalist Software License][acsl] (v. 1.4).

[acsl]: https://anticapitalist.software/
[archive-today]: https://archive.today/
[cancel-culture]: https://github.com/travisbrown/cancel-culture
[gojq]: https://github.com/itchyny/gojq
[internet-archive]: https://archive.org/
[ndjson]: http://ndjson.org/
[twitter-stream-grab]: https://archive.org/details/twitterstream
[user-object]: https://developer.twitter.com/en/docs/twitter-api/v1/data-dictionary/object-model/user
[wayback-machine]: https://archive.org/web/
