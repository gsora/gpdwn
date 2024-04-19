use gumdrop::Options;
mod gp;

#[derive(gumdrop::Options)]
struct CLI {
    #[options(help = "only download media IDs of \"video\" kind")]
    video_only: bool,

    #[options(
        help = "download everything off GoPro servers ,even if they don't appear as media on their API"
    )]
    everything: bool,

    #[options(
        required,
        help = "authentication token from https://plus.gopro.com, found  in your browser's \"gp_access_token\" cookie"
    )]
    auth_token: String,

    #[options(
        default = "25",
        help = "how many media IDs to download for each download URL generated, setting this to 0 will generate a single link with all media IDs"
    )]
    chunk_size: usize,

    #[options(help = "print usage")]
    help: bool,
}

fn main() {
    env_logger::init();

    let args = CLI::parse_args_default_or_exit();

    let handle = gp::Handle::new(&args.auth_token);

    let media_ids = match handle.media_ids(args.everything, args.video_only) {
        Ok(ids) => ids,
        Err(e) => {
            log::error!("error: {}", e);
            std::process::exit(1);
        }
    };

    for chunk in media_ids.chunks(args.chunk_size) {
        let du = match handle.download_url(chunk.to_vec()) {
            Ok(du) => du,
            Err(e) => {
                log::error!("error: {}", e);
                std::process::exit(1);
            }
        };

        println!("{}", du);
    }
}
