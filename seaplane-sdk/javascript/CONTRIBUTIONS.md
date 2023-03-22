# Contributing to `Seaplane Javascript/Typescript SDK`

Contributions are welcome!

We maintain a Seaplane SDK in both Javascript and Typescript modules.

## Dependencies

Install dependencies

```
$ npm install seaplane
```

## Packaging

To package the project as a source distribution:

```
$ npm run build
```

## Enforcing Code Quality

Automated code quality checks are performed using [eslint](https://eslint.org/)

```
$ npm run lint
```

## Testing

```
$ npm run test
```

## Automated Code Formatting

```bash
$ npm run pretty
```

## Publish a new version

> ⚠️ Only approved contributors can push tags to trigger a new release.

You have to create a new tag, adding the version of your package.

The version in `package.json` has to be bigger than the published package. You can increase the version using npm, using, `major.minor.patch`.

```
npm version patch 
```

The tag convention is `sdk-js-v*` being `*` the version of the SDK.

```
git tag -a sdk-js-v1.0 -m "Javascript SDK v1.0 Release"
```