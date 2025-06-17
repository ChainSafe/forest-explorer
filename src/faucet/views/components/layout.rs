use leptos::prelude::*;
use leptos::{IntoView, component, view};

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="space-y-6 flex flex-col items-center">
            <h1 class="header-large">Filecoin Forest Explorer Faucet</h1>
            <p class="max-w-2xl text-center">
                The Filecoin Forest Explorer Faucet provides developers and users with free calibnet(tFil) and mainnet(FIL) to support their exploration, testing and development on the Filecoin network.
            </p>
        </header>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-content">
                <div class="footer-section">
                    <strong class="footer-title">Learn More</strong>
                    <ul class="footer-links">
                        <li>
                            <a
                                href="https://docs.filecoin.io/networks/calibration/"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link-text"
                            >
                                Calibration Network Documentation
                            </a>
                        </li>

                    </ul>
                </div>

                <div class="footer-section">
                    <strong class="footer-title">Feedback & Support</strong>
                    <ul class="footer-links">
                        <li>
                            <a
                                href="https://github.com/ChainSafe/forest-explorer/issues"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link-text"
                            >
                                Report Issues on GitHub
                            </a>
                        </li>
                        <li>
                            <a
                                href="https://github.com/ChainSafe/forest-explorer/discussions"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link-text"
                            >
                                Suggest Improvements
                            </a>
                        </li>
                        <li>
                            <a
                                href="https://github.com/ChainSafe/forest-explorer"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link-text"
                            >
                                Contribute to the Project
                            </a>
                        </li>
                    </ul>
                </div>
            </div>

            <div class="footer-bottom">
                <span>
                    <a
                        class="text-green-700"
                        target="_blank"
                        rel="noopener noreferrer"
                        href="https://github.com/ChainSafe/forest-explorer"
                    >
                        Forest Explorer
                    </a>
                    ", built with ❤️ by "
                    <a class="text-blue-600" target="_blank" rel="noopener noreferrer" href="https://chainsafe.io">
                        ChainSafe Systems
                    </a>
                </span>
            </div>
        </footer>
    }
}
