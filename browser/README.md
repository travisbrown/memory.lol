# memory.lol browser extension

This browser extension accesses data from the [memory.lol][memory.lol] web service,
which means that a third party (me, Travis Brown) could in theory keep track of every
Twitter profile you view while using this extension. I am not logging this data, and the extension
does not communicate your screen name or other information about your account to the service. If you
consider information about which Twitter profiles you view to be sensitive, though, you probably
should not use this service.

There are also no guarantees about the availability of the service,
and access to account histories is currently limited for untrusted users
(please see the [project documentation for details][restrictions]).

Please also note that this software is **not** "open source",
but the source is available for use and modification by individuals, non-profit organizations, and worker-owned businesses
(see the [license section](#license) below for details).

## Installation

The extension must currently be built from source, which requires `git` and [`npm`][npm-installation].

```bash
$ git clone https://github.com/travisbrown/memory.lol.git
$ cd memory.lol/browser/
$ npm install
$ npm run build
```

### Chrome

Open `chrome://extensions/` in your browser, click "Load unpacked", and choose the `dist/` directory
from the `memory.lol/browser` directory where you just ran the `npm` commands.

### Firefox

Open `about:debugging#/runtime/this-firefox`, click "Load Temporary Add-on...", and choose the
`manifest.json` file in the `dist/` directory.
The extension will be available until the next time you restart the browser.

### Uninstalling

To uninstall on either browser, you can navigate to the page where you installed the extension and click
"Remove". On Firefox you can also uninstall the extension by restarting your browser.

## License

This software is published under the [Anti-Capitalist Software License][acsl] (v. 1.4).

[acsl]: https://anticapitalist.software/
[memory.lol]: https://github.com/travisbrown/memory.lol/
[npm-installation]: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
[restrictions]: https://github.com/travisbrown/memory.lol/#current-access-restrictions
