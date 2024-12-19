<a name="unreleased"></a>
## [Unreleased]

### Bug Fixes
1. **Twitch Websocket:** Handling reconnect the same as token elapsed. - [c6be250](https://github.com/JDPlays-Madhouse/RCON2.0/commit/c6be250793490346dc1c379b281e1f4c1ad4dadb?w=1&diff=split)

### Development
1. Added separate token cache for development. - [ccd5aa6](https://github.com/JDPlays-Madhouse/RCON2.0/commit/ccd5aa6757c8fa2b8c49f2a0de7365262c99e000?w=1&diff=split)


<a name="0.1.8"></a>
## [0.1.8](https://github.com/JDPlays-Madhouse/RCON2.0/compare/0.1.7...0.1.8) (2024-12-18)

### Bug Fixes

1. **Twitch OAuth:** Removed early refresh of token. - [7750d3e](https://github.com/JDPlays-Madhouse/RCON2.0/commit/7750d3eb8c2a41c292c11c0508a1c0ca93dc91bc?w=1&diff=split)
1. **Twitch OAuth:** Added validation to refresh. - [8356914](https://github.com/JDPlays-Madhouse/RCON2.0/commit/8356914507a3d94490d6d10c1331a4a533c0d4e6?w=1&diff=split)

### Features

1. **UI:** Changed over to component based routing. - [1e1fb57](https://github.com/JDPlays-Madhouse/RCON2.0/commit/1e1fb57b2db8e36faa51c8c866c420dc56206b63?w=1&diff=split)


<a name="0.1.7"></a>
## [0.1.7](https://github.com/JDPlays-Madhouse/RCON2.0/compare/0.1.6...0.1.7) (2024-12-17)

### Bug Fixes

1. **CICD:** Auto changelog removed - [c584304](https://github.com/JDPlays-Madhouse/RCON2.0/commit/c584304a1a52f6a6eb0bcd556df2ace8d0fb9da6?w=1&diff=split)
1. **CICD:** Auto changelog removed - [1acfbbd](https://github.com/JDPlays-Madhouse/RCON2.0/commit/1acfbbd4002bfa7a84f278a1c99a0b0246560e20?w=1&diff=split)
1. **RCON Server:** Multiple connections when reload page. - [36a1dea](https://github.com/JDPlays-Madhouse/RCON2.0/commit/36a1dea7399ff4de4a38146379e57b681306b5f9?w=1&diff=split)
1. **Twitch OAuth:** Changed to a common refresh function. - [1d27cab](https://github.com/JDPlays-Madhouse/RCON2.0/commit/1d27cabf5bae7b1be44b49117a9cef9af28fab57?w=1&diff=split)
1. **ui:** Changed to Pagination Componant - [38b4f77](https://github.com/JDPlays-Madhouse/RCON2.0/commit/38b4f772464adf537ec94a963b3014f92cda87de?w=1&diff=split)

### Debugging Measure

1. **Twitch OAuth Token:** Added extra logging to see details about suspected error. - [e2c24b9](https://github.com/JDPlays-Madhouse/RCON2.0/commit/e2c24b9317394a18414df39f6ffa2c4c62f4b660?w=1&diff=split)

### Documentation

1. Modified template to use ordered list. - [1c41448](https://github.com/JDPlays-Madhouse/RCON2.0/commit/1c41448dde4488d4fae6959aed8316fe97b31f52?w=1&diff=split)
1. Added subscription and modified reward redemption. - [1495ccf](https://github.com/JDPlays-Madhouse/RCON2.0/commit/1495ccf8aa772eb97b527c4a7783e9be21bb0759?w=1&diff=split)
1. Added commit links to changelog to easily see code changes. - [5c028a2](https://github.com/JDPlays-Madhouse/RCON2.0/commit/5c028a2d14d71da7a39b5d0f695d6348ba6bd0fe?w=1&diff=split)

### Features

1. **Channel Point Reward:** Added clearer logging of rewards. - [89ec338](https://github.com/JDPlays-Madhouse/RCON2.0/commit/89ec338d4bfd2b2e35b521dce32bf3cddf43f537?w=1&diff=split)
1. **Twitch Triggers:** Channel Subscription. - [9196694](https://github.com/JDPlays-Madhouse/RCON2.0/commit/919669420b6e9ea51ebaf0e34643eef2e709f45c?w=1&diff=split)
1. **Twitch Websocket:** Uses common refresh function in run function. - [e8ac0e7](https://github.com/JDPlays-Madhouse/RCON2.0/commit/e8ac0e733ee0a11c4573a57bcd314aa49c83e9b7?w=1&diff=split)
1. **UI:** Page Channel Point Rewards added with ability to copy required text for config. - [6119a1a](https://github.com/JDPlays-Madhouse/RCON2.0/commit/6119a1ada3edd6b7489c21386595621136a0d39e?w=1&diff=split)
1. **Websocket Event:** Handling both new and updated triggers. - [f9fbd71](https://github.com/JDPlays-Madhouse/RCON2.0/commit/f9fbd71bf570ed80885a522e98a38d311af74d92?w=1&diff=split)


<a name="0.1.6"></a>
## [0.1.6](https://github.com/JDPlays-Madhouse/RCON2.0/compare/0.1.5...0.1.6) (2024-12-13)

### Bug Fixes

1. name changed to title in trigger. - [a0023cb](https://github.com/JDPlays-Madhouse/RCON2.0/commit/a0023cb5ea7b9bc531161d0daa5417a504030414?w=1&diff=split)
1. **Server Config Form:** So the current data will auto fill. - [6772d79](https://github.com/JDPlays-Madhouse/RCON2.0/commit/6772d7954ae4d158fa7568226a60547676a2a593?w=1&diff=split)
1. **UI:** Changed to pre-defined names for the tooltips. - [1164ce2](https://github.com/JDPlays-Madhouse/RCON2.0/commit/1164ce286e6334619fe45761d9256a1367c046a1?w=1&diff=split)

### Code Refactoring

1. Removed fake data used for table. - [7c9ff26](https://github.com/JDPlays-Madhouse/RCON2.0/commit/7c9ff265abb793833ce33f4d004ee4de740ce2fc?w=1&diff=split)

### Documentation

1. Added documentation for scripts. - [412e016](https://github.com/JDPlays-Madhouse/RCON2.0/commit/412e01606d945ab57c6f8fa77b94f4d3701f65f8?w=1&diff=split)


<a name="0.1.5"></a>
## [0.1.5](https://github.com/JDPlays-Madhouse/RCON2.0/compare/0.1.4...0.1.5) (2024-12-12)

### Features

1. Displaying commands and triggers on the ui. - [ade46c7](https://github.com/JDPlays-Madhouse/RCON2.0/commit/ade46c78516a47a678a23d2496bf94b72380138d?w=1&diff=split)
1. added icons for integration status - [4767a01](https://github.com/JDPlays-Madhouse/RCON2.0/commit/4767a01ec0ed0231c18c34f0d578861d6befc4ca?w=1&diff=split)
1. Commands now react to triggers. - [d956b68](https://github.com/JDPlays-Madhouse/RCON2.0/commit/d956b68576e3e17a0602e78966628f1c4dd0c01a?w=1&diff=split)
1. added listing integrations for menus - [7c2dce9](https://github.com/JDPlays-Madhouse/RCON2.0/commit/7c2dce96ffda277e3509b05085b46dd7a4d37199?w=1&diff=split)
1. **commands:** working pathway for commands - [44e5a94](https://github.com/JDPlays-Madhouse/RCON2.0/commit/44e5a94d2890c19061cbfeb8c2f681bcd6ae8128?w=1&diff=split)

### Pull Requests

1. Merge pull request [#1](https://github.com/JDPlays-Madhouse/RCON2.0/issues/1) from JDPlays-Madhouse/dependabot/npm_and_yarn/cross-spawn-7.0.6


<a name="0.1.4"></a>
## [0.1.4](https://github.com/JDPlays-Madhouse/RCON2.0/compare/0.1.3...0.1.4) (2024-11-13)

### Bug Fixes

1. **twitch:** Added detection for websocket timeout. - [573dd92](https://github.com/JDPlays-Madhouse/RCON2.0/commit/573dd92006d005e3ecc7270ce7bc8c9b466822b5?w=1&diff=split)


<a name="0.1.3"></a>
## [0.1.3](https://github.com/JDPlays-Madhouse/RCON2.0/compare/0.1.2...0.1.3) (2024-11-08)

### Bug Fixes

1. **twitch:** removed the thread spawn. - [1d3a56c](https://github.com/JDPlays-Madhouse/RCON2.0/commit/1d3a56c39ff86ea1caa1fc0e3bdc96a3fa2bf9ab?w=1&diff=split)


<a name="0.1.2"></a>
## [0.1.2](https://github.com/JDPlays-Madhouse/RCON2.0/compare/0.1.1...0.1.2) (2024-11-08)

### Bug Fixes

1. **cicd:** fixing caching - [7990a97](https://github.com/JDPlays-Madhouse/RCON2.0/commit/7990a972183675752886474531058ed52e40cbe9?w=1&diff=split)
1. **cicd:** caching added - [3ec933f](https://github.com/JDPlays-Madhouse/RCON2.0/commit/3ec933ff5c6ebec479c52c025cd948a463d0bedd?w=1&diff=split)
1. **cicd:** forced build to be tested first. - [9c97d24](https://github.com/JDPlays-Madhouse/RCON2.0/commit/9c97d2495707960ac8fa16484a3915e443150939?w=1&diff=split)


<a name="0.1.1"></a>
## [0.1.1](https://github.com/JDPlays-Madhouse/RCON2.0/compare/0.1.0...0.1.1) (2024-11-06)

### Bug Fixes

1. **cicd:** turned on tauri bundling - [1626694](https://github.com/JDPlays-Madhouse/RCON2.0/commit/1626694fb402c4ff32cf4ca34160bd98562332af?w=1&diff=split)
1. **cicd:** turned on tauri bundling - [55dbf2d](https://github.com/JDPlays-Madhouse/RCON2.0/commit/55dbf2de9f931fe41c13d941cbd3315a935a5b7c?w=1&diff=split)
1. **cicd:** added working directory to tests - [701d55a](https://github.com/JDPlays-Madhouse/RCON2.0/commit/701d55a8884a02d0891f39784f1e8672707bfd51?w=1&diff=split)
1. **cicd:** added a cd to src-tauri - [3f43285](https://github.com/JDPlays-Madhouse/RCON2.0/commit/3f432858eaa01815ed187d296de0071263e24532?w=1&diff=split)
1. **cicd:** fix to env variables - [c2706e2](https://github.com/JDPlays-Madhouse/RCON2.0/commit/c2706e2d6e57416f1439f5685cfe9ff536cf9ef9?w=1&diff=split)
1. **cicd:** manual_dispatch to workflow_dispatch - [e770482](https://github.com/JDPlays-Madhouse/RCON2.0/commit/e77048274f4eef64d1d301724e287b468aeb9de9?w=1&diff=split)
1. **lib:** bug in tauri thread spawn, changed to tokio. - [3155bed](https://github.com/JDPlays-Madhouse/RCON2.0/commit/3155bed7142102b669133e4ff2af0fcaa48e97ee?w=1&diff=split)


<a name="0.1.0"></a>
## 0.1.0 (2024-11-05)

### Features

1. **frontend:** Added shadcn and next-theme - [0826099](https://github.com/JDPlays-Madhouse/RCON2.0/commit/08260995c1304b40f418bfbe420d2258daa86c7e?w=1&diff=split)
1. **logging:** Logging now works as intended. - [bccc860](https://github.com/JDPlays-Madhouse/RCON2.0/commit/bccc860bc998f4b09c88a967a3d8f7a8db43f572?w=1&diff=split)

