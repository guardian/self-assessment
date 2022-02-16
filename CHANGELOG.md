# Change Log
All notable changes to this project will be documented in this file.
 
The format is based on [Keep a Changelog](http://keepachangelog.com/)
 
## [0.1.1] - 2022-02-16
 
### Added
- The tool now outputs descriptive status updates when running.
- The `GuardianPullRequests` enum now implements the `Display` trait.
 
### Changed
- In order to decrease the likelihood of hitting GitHub's secondary rate limit, the logic has been changed so that polling stops once the number of fetched items matches the number of the total items in the response. 
- The `Args` struct now lives in its own module (`cli.rs`).
- The `body` field in the `GithubSearchResponseItem` struct has been changed from a `String` type to an `Option<String>` type to account for empty PR descriptions.

### Fixed
- The new `body` type (`Option<String>`) means that execution no longer halts when encountering a pull request with an empty body.