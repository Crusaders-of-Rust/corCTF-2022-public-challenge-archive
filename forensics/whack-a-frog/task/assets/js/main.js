window.onload = () => {
// constants
const FROG_HEIGHT = 35
const FROG_WIDTH = 56

const FROG_IMAGES = ["img/rook.png", "img/normalcastle.png"]
const FROG_IMAGE_ELEMENTS = []

let selectedFrog = null
const frogSelectionPane = document.createElement("div")

const brushSelectionText = document.createElement("h2")
brushSelectionText.style.textAlign = "center"
 brushSelectionText.innerText = "Pick your frog brush"
frogSelectionPane.appendChild(brushSelectionText)

for (const frogImage of FROG_IMAGES) {
  const image = document.createElement("img")
  image.src = frogImage

  image.addEventListener("click", event => {
    // Add outline to show that it's selected
    for (const otherImage of FROG_IMAGE_ELEMENTS) {
      otherImage.style.outline = "none"
    }
    image.style.outline = "2px solid red"
    selectedFrog = frogImage
  })
  FROG_IMAGE_ELEMENTS.push(image)
}

FROG_IMAGE_ELEMENTS[1].dispatchEvent(new Event("click"))

document.body.style.fontFamily = `"Comic Sans MS", "Comic Sans", cursive`
document.body.style.display = "flex"
document.body.style.flexDirection = "row"
FROG_IMAGE_ELEMENTS.forEach(n => frogSelectionPane.appendChild(n))

const frogHammerText = document.createElement("h2")
frogHammerText.style.textAlign = "center"
frogHammerText.innerText = "Click and drag to apply frog hammer.  Our patented server-side Pluton DRM will stop illegal reproduction of movies in frog art whilst still ensuring that your drawings should remain secure "
frogSelectionPane.appendChild(frogHammerText)
const fsf = document.createElement("img")
fsf.src = "img/frog_software_foundation.png"
frogSelectionPane.appendChild(fsf)


// create frog_widthxfrog_height 2d array
let pixelStates = new Array(FROG_HEIGHT);
for (let i = 0; i < FROG_HEIGHT; i++) {
  pixelStates[i] = new Array(FROG_WIDTH);
  for (let j = 0; j < pixelStates.length; j++) {
    pixelStates[i][j] = true
  }
}

let frogPreviewPane = document.createElement("img")
let tableElement = document.createElement("table");
// Create a 56x56 table of rook images

let isDragging = false
document.addEventListener("mousedown", () => isDragging = true)
document.addEventListener("mouseup", () => isDragging = false)


for (let i = 0; i < FROG_HEIGHT; i++) {
  let row = tableElement.insertRow(i);
  for (let j = 0; j < FROG_WIDTH; j++) {
    const cell = row.insertCell(j);
    const img = document.createElement("img");
    img.style.padding = "0rem";
    img.style.margin = "0"
    img.src = "img/rook.png"
    img.width = 16
    img.height = 14;
    pixelStates[i][j] = [true, false];

    const updatePreviewPane = () => {
      frogPreviewPane.src = img.src;
      frogPreviewPane.width = img.width / 14;
      frogPreviewPane.height = img.height / 14;
    };
    const maybeToggleFrog = () => {
      if (isDragging && selectedFrog) {
        img.src = selectedFrog;
        console.log("changed image source to " + img.src)
        pixelStates[i][j] = selectedFrog
        updatePreviewPane();
      }
    }

    img.addEventListener("mouseover", maybeToggleFrog)

    img.addEventListener("mouseenter", updatePreviewPane)
    cell.appendChild(img);
  }
}

document.body.addEventListener("mousemove", async event => {
  await fetch(`/anticheat?x=${event.clientX}&y=${event.clientY}&event=mousemove`)
});
document.body.addEventListener("mouseup", async event => {
  await fetch(`/anticheat?x=${event.clientX}&y=${event.clientY}&event=mouseup`)
});
document.body.addEventListener("mousedown", async event => {
  await fetch(`/anticheat?x=${event.clientX}&y=${event.clientY}&event=mousedown`)
});


document.body.appendChild(tableElement);
document.body.appendChild(frogPreviewPane);
document.body.appendChild(frogSelectionPane)
}
