# y1

y1 is our task runner that pretends to be `yarn@1.22.22`,
but only does task running.

We use it because it's supported by [turborepo](https://turborepo.dev/docs/getting-started/add-to-existing-repository#add-a-packagemanager-field-to-root-packagejson)
and we don't want to mix task running and dependency management. Our workspace package.json says:

```
  "packageManager": "yarn@1.22.22",
```

## Task runner only

LLMs and developers alike will typically try to manage dependencies using the declared packageManager.
We need to be a lot more flexible with dependency management than with task running.

## Performance

We run hundreds of thousands of tasks per day. Here's yarn's cost for a trivial task on a regular macbook (using nodejs):

```
% time yarn run test
yarn run v1.22.22
$ true
✨  Done in 0.02s.
yarn run test  0.11s user 0.03s system 98% cpu 0.141 total
```

## Reliable signals

We need a way to catch the result of every single task through logs.
It's great that yarn v1 prints the time taken for a task.
It's also great that it prints the exit code on error.

## Additional args

We emulate yarn v1's r,
but also support single double dashes.

## Configuration

y1 does not read yarn config.
It's meant to behave the same on any system.
It's basically unaffected by env.

## Supported subcommands

* `yarn run`
  - Runs as: `yarn run --non-interactive`

* `yarn run [task]`

  - Omits: stdout line "info Visit https://yarnpkg.com/en/docs/cli/run for documentation about this command."
  - Accepts `-- --` as separator for custom args ([the original behavior](https://github.com/vercel/turborepo/blob/v2.8.20/apps/docs/content/docs/reference/run.mdx?plain=1#L34))
  - Deviation: Also accepts `--` with the same effect as `-- --` to work like other task runners.

* `yarn [task]`

  - like `yarn [task]` except for any subcommand listed in yarn@1.22.22's help.

* `yarn help` | `yarn --help` | `yarn -h`

  - see [Help section](#help-section)

* `yarn help run`

  - see [yarn help run](#yarn-help-run)

## Notably rejected

Every invocation that isn't documented as supported is an error.

The error message is: `y1 yarn port rejected: $@`

The exit code is: 99

* `yarn` without args in the actual yarn v1 does dependency management. The error message here is `y1 yarn port requires a subcommand`. Exit code is 99 here too.

## Help section

Similar to the original help but:

  - Omits: every line that's an unsupported flag or subcommand.
  - Omits: "Visit https://yarnpkg.com/en/docs/cli/ to learn more about Yarn."
  - Adds: a line identifying the y1 tool with version.

### yarn help run

Samp principle as [Help section](#help-section).

## Development tooling

A CLI written in latest Rust.

Multi-arch builds (linux + mac and arm64 + amd64) using github action with pegged action versions.

Releases on tags using github actions, with uncompressed single binary downloads and a checksum file.

## Running original yarn during developmen

For example: `npm run refresh >/dev/null && (cd examples/minimal; ../../node_modules/.bin/yarn test)`
