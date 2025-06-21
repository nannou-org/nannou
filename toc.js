// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="welcome.html">Welcome!</a></li><li class="chapter-item expanded "><a href="why_nannou.html"><strong aria-hidden="true">1.</strong> Why Nannou?</a></li><li class="chapter-item expanded "><a href="getting_started.html"><strong aria-hidden="true">2.</strong> Getting Started</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="getting_started/platform-specific_setup.html"><strong aria-hidden="true">2.1.</strong> Platform-specific Setup</a></li><li class="chapter-item expanded "><a href="getting_started/installing_rust.html"><strong aria-hidden="true">2.2.</strong> Installing Rust</a></li><li class="chapter-item expanded "><a href="getting_started/editor_setup.html"><strong aria-hidden="true">2.3.</strong> Editor Setup</a></li><li class="chapter-item expanded "><a href="getting_started/running_examples.html"><strong aria-hidden="true">2.4.</strong> Running Examples</a></li><li class="chapter-item expanded "><a href="getting_started/create_a_project.html"><strong aria-hidden="true">2.5.</strong> Create A Project</a></li><li class="chapter-item expanded "><a href="getting_started/updating.html"><strong aria-hidden="true">2.6.</strong> Updating Nannou and Rust</a></li></ol></li><li class="chapter-item expanded "><a href="tutorials.html"><strong aria-hidden="true">3.</strong> Tutorials</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="tutorials/basics/anatomy-of-a-nannou-app.html"><strong aria-hidden="true">3.1.</strong> Basics - Anatomy of a Nannou App</a></li><li class="chapter-item expanded "><a href="tutorials/basics/draw-a-sketch.html"><strong aria-hidden="true">3.2.</strong> Basics - Drawing a Sketch</a></li><li class="chapter-item expanded "><a href="tutorials/basics/sketch-vs-app.html"><strong aria-hidden="true">3.3.</strong> Basics - Sketch vs App</a></li><li class="chapter-item expanded "><a href="tutorials/basics/window-coordinates.html"><strong aria-hidden="true">3.4.</strong> Basics - Window Coordinates</a></li><li class="chapter-item expanded "><a href="tutorials/draw/drawing-2d-shapes.html"><strong aria-hidden="true">3.5.</strong> Draw - 2D Shapes</a></li><li class="chapter-item expanded "><a href="tutorials/draw/animating-a-circle.html"><strong aria-hidden="true">3.6.</strong> Draw - Animating a Circle</a></li><li class="chapter-item expanded "><a href="tutorials/draw/drawing-images.html"><strong aria-hidden="true">3.7.</strong> Draw - Drawing Images</a></li><li class="chapter-item expanded "><a href="tutorials/osc/osc-introduction.html"><strong aria-hidden="true">3.8.</strong> OSC - Introduction</a></li><li class="chapter-item expanded "><a href="tutorials/osc/osc-sender.html"><strong aria-hidden="true">3.9.</strong> OSC - Sending OSC</a></li></ol></li><li class="chapter-item expanded "><a href="community_tutorials.html"><strong aria-hidden="true">4.</strong> Community Tutorials</a></li><li class="chapter-item expanded "><a href="contributing.html"><strong aria-hidden="true">5.</strong> Contributing</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="contributing/about-the-codebase.html"><strong aria-hidden="true">5.1.</strong> About the Codebase</a></li><li class="chapter-item expanded "><a href="contributing/pr-checklist.html"><strong aria-hidden="true">5.2.</strong> PR Checklist</a></li><li class="chapter-item expanded "><a href="contributing/pr-reviews.html"><strong aria-hidden="true">5.3.</strong> PR Reviews</a></li><li class="chapter-item expanded "><a href="contributing/publishing-new-versions.html"><strong aria-hidden="true">5.4.</strong> Publishing New Versions</a></li></ol></li><li class="chapter-item expanded "><a href="developer_reference.html"><strong aria-hidden="true">6.</strong> Developer Reference</a></li><li class="chapter-item expanded "><a href="api_reference.html"><strong aria-hidden="true">7.</strong> API Reference</a></li><li class="chapter-item expanded "><a href="showcases.html"><strong aria-hidden="true">8.</strong> Showcases</a></li><li class="chapter-item expanded affix "><a href="changelog.html">Changelog</a></li><li class="chapter-item expanded affix "><a href="contributors.html">Contributors</a></li><li class="chapter-item expanded affix "><a href="code_of_conduct.html">Code of Conduct</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
