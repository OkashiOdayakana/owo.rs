# OwO.rs

An uploader for the [OwO](https://whats-th.is) file-hosting CDN.

It also works as a library, although not yet recommended, as the API is (currently) unstable.

## Usage
The CLI has two functions: uploading, and shortening.

Currently, proper config files are not supported yet, 
so the configuration options are managed via either command line flags, or environmental variables.

### Upload
`OWO_KEY=KEY owo file.jpg`

### Shorten
`OWO_KEY=KEY owo shorten https://google.com`
