# Maintainers

## Cutting a New Release

* Update the [CHANGELOG]:
  * Create a new branch to prepare the release.

  * Ensure the [CHANGELOG] is up to date. (See
    [below][changelog-refresh] for a suggested workflow.)

  * Retitle the "Unreleased" section to this release and create a fresh
    "Unreleased" section (see comments in the [CHANGELOG] for details).

    Do not wrap bullet points in multiple lines. GitHub will use those line
    breaks in the Release Notes display.

    :bulb: Point releases (i.e., not patch releases) should also be
    given a name, taking the form `ADJECTIVE TREE`, incrementing
    alphabetically. This name should be decided amongst the team before
    the release.

  * Bump the package version number in `Cargo.toml` at project root.

  * Commit and merge (squash, if necessary) on green CI and peer
    approval.

  * Tag the merged commit with the release version, prefixed with a `v` (e.g.,
    `v0.1.0`). The version number must match the one in `Cargo.toml`, otherwise
    `cargo dist` will fail during CI.

    ```bash
    git tag "v0.1.0"
    git push
    git push --tags
    ```

* Let `cargo dist` create a new [draft release][releases].
  * Verify the release.
  * Publish the draft release.
  * If all went well, consider if we should let [CI publish the draft
    automatically][auto-publish].

* Publicise.

  :warning: Point releases, only. Don't publicise patch releases, unless
  there's a pressing need to do so (e.g., fix of a security
  vulnerability, etc.).

  * Announce the new version on Tweag's Twitter and other social network
    accounts, via someone with access.
  * Share amongst other social networks (e.g., Reddit, Hacker News,
    Mastodon, etc.), under personal accounts, at your discretion.

### Generating the PR List for the CHANGELOG

If the unreleased changes in the [CHANGELOG] have become stale, the list
of merged PRs can be fetched from:

    https://github.com/tweag/topiary/pulls?q=is:pr+base:main+merged:>YYYY-MM-DD

Replacing `YYYY-MM-DD` by the date of the last release.

If you have the GitHub CLI client, the following may be more convenient:

```bash
gh pr list -L 500 -B main -s merged \
           --json number,mergedAt,title,body \
| jq -r --argjson release "$(gh release view --json createdAt)" '
     reverse | .[] | select(.mergedAt > $release.createdAt) |
     ["# PR#\(.number): \(.title)", "*Merged: \(.mergedAt)*", "\(.body)\n"] |
     join("\n\n")'
```

:bulb: The `-L 500` is an arbitrary "large number" limit of PRs to
fetch, overriding the low default. As of writing, there's no way to set
this to "unlimited"; adjust as necessary.

<!-- Links -->
[changelog]: /CHANGELOG.md
[changelog-refresh]: #generating-the-pr-list-for-the-changelog
[releases]: https://github.com/tweag/topiary/releases
[auto-publish]: https://github.com/tweag/topiary/pull/538/commits/230d7bc662a042188c79d586472c4e8632ffd6a9
