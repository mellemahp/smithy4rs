# Introduction

Thank you for your interest in contributing to `smithy4rs`. It's people like you that 
help the open source ecosystem around Smithy thrive.

Following these guidelines helps to communicate that you respect the time of the developers managing and developing this open source project. 
In return, the core developers will reciprocate that respect in addressing your issue, assessing changes, and helping you finalize your pull requests.


### What kind of contributions are we looking for?

`smithy4rs` is an open source, community-supported project, and greatly appreciate contributions from the broader Smithy community — you! 
There are many ways to contribute, from writing blog posts or tutorials, improving the documentation, submitting bug reports and 
feature requests or writing code which can be incorporated into `smithy4rs` itself.

### What kind of contributions are we NOT looking for

Please, don't use the issue tracker for issues relating to the core Smithy toolchain or the AWS Rust SDK. `smithy4rs` is **not**
affiliated with AWS and such issues can be directed to the official Smithy repos (see: [smithy-lang](https://github.com/smithy-lang)).

Also, while we are not opposed to LLM-assisted contributions, we expect contributors to thoroughly review such contributions 
to ensure they are high-quality, hallucination-free, and consistent with the style/mental-model of the rest of the repository.

# Ground Rules

1. Be respectful and considerate.
2. Be welcoming to newcomers and encourage diverse new contributors from all backgrounds.
3. Create issues for any major changes and enhancements that you wish to make. Discuss things transparently and get feedback.
4. Ensure cross-platform compatibility for every change that's accepted. Windows, Mac, Debian & Ubuntu Linux.

# Security Issues

*Never* report security related issues, vulnerabilities or bugs including sensitive information to the bug tracker, or elsewhere in public. 
Instead, email: `contact@scaffold-api.com` with a detailed description of the issue.

# How to report a bug
> [!WARNING]
> Do *NOT* report security issues to the issue tracker. See: [Security Issues](#security-issues).

When filing an issue, make sure to answer these four questions:

1. What operating system and processor architecture are you using?
2. What did you do?
3. What did you expect to see?
4. What did you see instead?

> [!NOTE]
> For general questions, use the [GitHub discussions](https://github.com/mellemahp/smithy4rs/discussions/landing)

# How to suggest a feature or enhancement
The `smithy4rs` philosophy is to provide small, robust tooling for protocol-agnostic clients, servers, and other 
rpc-like tools.

If you find yourself wishing for a feature that doesn't exist in `smithy4rs`, open an issue on our issues list on 
GitHub which describes: 
1. the feature you would like to see
2. why you need it
3. How you feel it should work.

# Contributions 
To submit a PR: 
1. Create your own fork of the code
2. Do the changes in your fork
3. If you like the change and think the project could use it:
   * Be sure your code passes the test run by `mise pre-push` command.
   * Send a pull request 
   * Address any issues found by Continuous integration workflows
   
# Code review process

The core maintainers look at Pull Requests on a regular basis (~1/week). 
After feedback has been given we expect responses within two weeks. 
After two weeks we may close the pull request if it isn't showing any activity.
