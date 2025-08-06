
function initMobilePanel() {
    const panel = document.getElementById('panel');
    const isMobile = window.innerWidth <= 767;

    if (isMobile) {



// Panel listener - only opens when closed
        document.addEventListener('click', (e) => {
            console.log('Document clicked');

            if (panel.contains(e.target)) {
                // Click inside panel
                console.log('Click inside panel');
                if (!panel.classList.contains('open')) {
                    console.log('Opening panel');
                    panel.classList.add('open');
                    savePanelState('true');
                }
                // Don't close when clicking inside - let buttons work normally
            } else {
                // Click outside panel
                console.log('Click outside panel');
                if (panel.classList.contains('open')) {
                    console.log('Closing panel');
                    panel.classList.remove('open');
                    savePanelState('false');
                }
            }
        });


        // Optional: Add swipe gestures
        let startY = 0;
        let currentY = 0;

        panel.addEventListener('touchstart', (e) => {
            startY = e.touches[0].clientY;
        });

        panel.addEventListener('touchmove', (e) => {
            currentY = e.touches[0].clientY;
        });

        panel.addEventListener('touchend', () => {
            const deltaY = startY - currentY;

            // Swipe up to open
            if (deltaY > 50 && !panel.classList.contains('open')) {
                panel.classList.add('open');
                savePanelState('true');
            }
            // Swipe down to close
            else if (deltaY < -50 && panel.classList.contains('open')) {
                panel.classList.remove('open');
                savePanelState('false');
            }
        });
    }
}
// Call on load and resize
window.addEventListener('load', initMobilePanel);
window.addEventListener('resize', initMobilePanel);



function togglePanel() {
    const panel = document.getElementById("panel");
    const button = document.getElementById("closeButton");

    if(panel.classList.contains("open")) {
        panel.classList.remove("open");
        button.innerText = ">";
        savePanelState('false');
    } else {
        panel.classList.add("open");
        button.innerText = "<";
        savePanelState('true');
    }

}
function savePanelState(open) {
    console.log("Saving panel state: " + open );
    sessionStorage.setItem('panelOpen', open);
}

function restorePanelState() {
    const saved = sessionStorage.getItem('panelOpen');
    setTimeout(() => {panel.classList.remove('no-transition');}, 10)
    if (saved == null) return;
    console.log('Restoring panel state:', saved);
    const wasOpen = saved === 'true';
    const panel = document.getElementById('panel');

    // Default: desktop open, mobile closed
    const isMobile = window.innerWidth <= 767;
    const shouldBeOpen = wasOpen !== null ? wasOpen : !isMobile;

    if (shouldBeOpen) {
        panel.classList.add('open');
    }
}




let content;
let container;
let isPanning = false;
let startX, startY, scrollLeft, scrollTop;
let X = 0;
let Y = -48.25;
let scale = 24;
//

window.onload = function () {
    const isMobile = window.innerWidth <= 767;
    const container = document.getElementById('container');
    const panzoom = Panzoom(container, {
        maxScale: 100,
        minScale: 1.5,
        startScale: 10,
        startX: 0,
        startY: -0.427439 * (23.9658 + window.innerHeight),
        contain: false,
        canvas: true,
        step: isMobile ? 4 : 0.5,
        pinchAndPan: false,
    });

    container.parentElement.addEventListener('wheel', panzoom.zoomWithWheel);
    document.getElementById('panel').addEventListener('wheel', function(event) {
        event.stopPropagation();
    });
    container.addEventListener('panzoomend', () => {
        const { x, y } = panzoom.getPan();

        let needsAdjustment = false;
        let newX = x, newY = y;

        // Set your actual boundaries here
        if (x < -500) { newX = -500; needsAdjustment = true; }
        if (x > 500) { newX = 500; needsAdjustment = true; }
        if (y < -500) { newY = -500; needsAdjustment = true; }
        if (y > 300) { newY = 300; needsAdjustment = true; }

        if (needsAdjustment) {
            panzoom.pan(newX, newY, { animate: true });
        }
    });
    let x = sessionStorage.getItem('positionX');
    let y = sessionStorage.getItem('positionY');
    let zoom = sessionStorage.getItem('zoom');
    if (x != null && y != null)
        setTimeout(() => {
            panzoom.pan(parseFloat(x), parseFloat(y));
        })
    if (zoom != null)
        setTimeout(() => {
            panzoom.zoom(parseFloat(zoom));
        })

    setInterval(() => {
        const {x, y} = panzoom.getPan();
        sessionStorage.setItem('positionX', x);
        sessionStorage.setItem('positionY', y);
        sessionStorage.setItem('zoom', panzoom.getScale());
    },500)
    restorePanelState()
};

Number.prototype.clamp = function(min, max) {
    return Math.min(Math.max(this, min), max);
};

function move() {
    const children = container.children // get the child elements
    for(const child of children){
        child.style.transform = "scale(" + scale + ") translateX(" + X + "%) translateY(" + Y + "%)"; // pass the scale to the childs only
    }
}

function IssueWindow(name) {
    const reportWindow = document.getElementById("reportWindow");
    const reportName = document.getElementById("reportName")
    const reportNameHidden = document.getElementById("reportNameHidden")
    reportWindow.classList.remove("hidden")
    reportName.innerHTML = name;
    reportNameHidden.setAttribute("value", name)
}

function CloseWindow() {
    const reportWindow = document.getElementById("reportWindow");
    reportWindow.classList.add("hidden");
}

Handlebars.registerHelper('loud', function (aString) {
    return aString.toUpperCase()
})
