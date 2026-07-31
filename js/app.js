/**
 * DocuShift - PDF to Word Converter Frontend JavaScript
 * Compatible with GitHub Pages & custom backend endpoints.
 */

document.addEventListener('DOMContentLoaded', () => {
  // Elements
  const dropzone = document.getElementById('dropzone');
  const fileInput = document.getElementById('file-input');
  const dropzoneCard = document.getElementById('dropzone-card');
  const previewCard = document.getElementById('preview-card');
  const progressCard = document.getElementById('progress-card');
  const successCard = document.getElementById('success-card');
  const mainGrid = document.getElementById('main-grid');

  const fileNameDisplay = document.getElementById('file-name-display');
  const fileSizeDisplay = document.getElementById('file-size-display');
  const btnChangeFile = document.getElementById('btn-change-file');

  const pdfCanvas = document.getElementById('pdf-canvas');
  const btnPrevPage = document.getElementById('btn-prev-page');
  const btnNextPage = document.getElementById('btn-next-page');
  const pageIndicator = document.getElementById('page-indicator');

  const startPageInput = document.getElementById('start-page-input');
  const endPageInput = document.getElementById('end-page-input');
  const btnConvertNow = document.getElementById('btn-convert-now');

  const progressStepText = document.getElementById('progress-step-text');
  const progressBarFill = document.getElementById('progress-bar-fill');

  const btnDownloadDocx = document.getElementById('btn-download-docx');
  const btnStartOver = document.getElementById('btn-start-over');

  const apiUrlInput = document.getElementById('api-url-input');
  const apiStatusIndicator = document.getElementById('api-status-indicator');
  const btnTestApi = document.getElementById('btn-test-api');

  // State Variables
  let currentFile = null;
  let pdfDoc = null;
  let currentPage = 1;
  let totalPages = 0;
  let docxBlobUrl = null;

  // Initialize PDF.js worker
  if (window.pdfjsLib) {
    pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js';
  }

  // Load saved API URL or default
  const savedApiUrl = localStorage.getItem('docushift_api_url');
  if (savedApiUrl) {
    apiUrlInput.value = savedApiUrl;
  }

  // Check Backend API Connectivity
  async function checkApiHealth() {
    const baseUrl = apiUrlInput.value.replace(/\/+$/, '');
    apiStatusIndicator.className = 'api-status-dot';
    apiStatusIndicator.title = 'Connecting...';

    try {
      const response = await fetch(`${baseUrl}/health`, { method: 'GET', mode: 'cors' });
      if (response.ok) {
        apiStatusIndicator.classList.remove('offline');
        apiStatusIndicator.title = 'Backend Connected';
      } else {
        apiStatusIndicator.classList.add('offline');
        apiStatusIndicator.title = 'Backend returned error';
      }
    } catch (err) {
      apiStatusIndicator.classList.add('offline');
      apiStatusIndicator.title = 'Backend unreachable (Check CORS / URL)';
    }
  }

  checkApiHealth();

  apiUrlInput.addEventListener('change', () => {
    localStorage.setItem('docushift_api_url', apiUrlInput.value.trim());
    checkApiHealth();
  });

  btnTestApi.addEventListener('click', checkApiHealth);

  // File Dropzone Events
  ['dragenter', 'dragover'].forEach(eventName => {
    dropzone.addEventListener(eventName, (e) => {
      e.preventDefault();
      e.stopPropagation();
      dropzone.classList.add('dragover');
    });
  });

  ['dragleave', 'drop'].forEach(eventName => {
    dropzone.addEventListener(eventName, (e) => {
      e.preventDefault();
      e.stopPropagation();
      dropzone.classList.remove('dragover');
    });
  });

  dropzone.addEventListener('drop', (e) => {
    const files = e.dataTransfer.files;
    if (files && files.length > 0) {
      handleFileSelection(files[0]);
    }
  });

  fileInput.addEventListener('change', (e) => {
    if (e.target.files && e.target.files.length > 0) {
      handleFileSelection(e.target.files[0]);
    }
  });

  function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  // Handle selected PDF file
  async function handleFileSelection(file) {
    if (file.type !== 'application/pdf' && !file.name.toLowerCase().endsWith('.pdf')) {
      alert('Please select a valid PDF file.');
      return;
    }

    currentFile = file;
    fileNameDisplay.textContent = file.name;
    fileSizeDisplay.textContent = formatBytes(file.size);

    // Show preview panel layout
    dropzoneCard.classList.add('hidden');
    previewCard.classList.remove('hidden');
    progressCard.classList.add('hidden');
    successCard.classList.add('hidden');
    mainGrid.classList.add('has-file');

    // Read and Render PDF preview via PDF.js
    const fileReader = new FileReader();
    fileReader.onload = async function () {
      const typedArray = new Uint8Array(this.result);
      try {
        pdfDoc = await pdfjsLib.getDocument({ data: typedArray }).promise;
        totalPages = pdfDoc.numPages;
        currentPage = 1;
        
        startPageInput.value = 1;
        endPageInput.value = totalPages;
        startPageInput.max = totalPages;
        endPageInput.max = totalPages;

        renderPdfPage(currentPage);
      } catch (err) {
        console.error('Error loading PDF:', err);
        alert('Could not parse PDF file preview.');
      }
    };
    fileReader.readAsArrayBuffer(file);
  }

  // Render Page on Canvas
  async function renderPdfPage(pageNum) {
    if (!pdfDoc) return;
    try {
      const page = await pdfDoc.getPage(pageNum);
      const viewport = page.getViewport({ scale: 1.2 });
      
      const context = pdfCanvas.getContext('2d');
      pdfCanvas.height = viewport.height;
      pdfCanvas.width = viewport.width;

      const renderContext = {
        canvasContext: context,
        viewport: viewport
      };

      await page.render(renderContext).promise;
      pageIndicator.textContent = `Page ${pageNum} of ${totalPages}`;
    } catch (err) {
      console.error('Error rendering page:', err);
    }
  }

  btnPrevPage.addEventListener('click', () => {
    if (currentPage > 1) {
      currentPage--;
      renderPdfPage(currentPage);
    }
  });

  btnNextPage.addEventListener('click', () => {
    if (currentPage < totalPages) {
      currentPage++;
      renderPdfPage(currentPage);
    }
  });

  btnChangeFile.addEventListener('click', resetUI);

  function resetUI() {
    currentFile = null;
    pdfDoc = null;
    if (docxBlobUrl) {
      URL.revokeObjectURL(docxBlobUrl);
      docxBlobUrl = null;
    }
    const docxContainer = document.getElementById('docx-container');
    if (docxContainer) docxContainer.innerHTML = '';

    fileInput.value = '';
    mainGrid.classList.remove('has-file');
    dropzoneCard.classList.remove('hidden');
    previewCard.classList.add('hidden');
    progressCard.classList.add('hidden');
    successCard.classList.add('hidden');
  }

  // Conversion process execution
  btnConvertNow.addEventListener('click', async () => {
    if (!currentFile) return;

    const baseUrl = apiUrlInput.value.replace(/\/+$/, '');
    const startVal = startPageInput.value ? parseInt(startPageInput.value) : 1;
    const endVal = endPageInput.value ? parseInt(endPageInput.value) : totalPages;

    // Switch to progress UI
    previewCard.classList.add('hidden');
    progressCard.classList.remove('hidden');

    let currentProgress = 10;
    progressBarFill.style.width = `${currentProgress}%`;
    progressStepText.textContent = 'Uploading PDF to engine...';

    const progressInterval = setInterval(() => {
      if (currentProgress < 85) {
        currentProgress += Math.floor(Math.random() * 8) + 3;
        progressBarFill.style.width = `${currentProgress}%`;
        
        if (currentProgress > 30 && currentProgress <= 55) {
          progressStepText.textContent = 'Detecting & redacting repeating headers/footers...';
        } else if (currentProgress > 55 && currentProgress <= 75) {
          progressStepText.textContent = 'Parsing PyMuPDF vector structures & tables...';
        } else if (currentProgress > 75) {
          progressStepText.textContent = 'Building Word (.docx) XML & adjusting bullet spacing...';
        }
      }
    }, 400);

    // Prepare FormData
    const formData = new FormData();
    formData.append('file', currentFile);
    formData.append('start', startVal);
    formData.append('end', endVal);

    try {
      const response = await fetch(`${baseUrl}/convert/pdf-to-docx`, {
        method: 'POST',
        mode: 'cors',
        body: formData
      });

      clearInterval(progressInterval);

      if (!response.ok) {
        let errMessage = 'Conversion failed.';
        try {
          const errJson = await response.json();
          if (errJson.error) errMessage = errJson.error;
        } catch (_) {}
        throw new Error(errMessage);
      }

      progressBarFill.style.width = '100%';
      progressStepText.textContent = 'Conversion successful!';

      const blob = await response.blob();
      docxBlobUrl = URL.createObjectURL(blob);

      const outName = currentFile.name.replace(/\.[^/.]+$/, "") + ".docx";
      btnDownloadDocx.href = docxBlobUrl;
      btnDownloadDocx.download = outName;

      // Render live DOCX document in browser
      const docxContainer = document.getElementById('docx-container');
      if (docxContainer) {
        docxContainer.innerHTML = '<div style="text-align:center; padding: 2rem; color: var(--text-muted);"><i class="fa-solid fa-spinner fa-spin"></i> Rendering document preview...</div>';
        if (window.docx && window.docx.renderAsync) {
          try {
            const arrayBuffer = await blob.arrayBuffer();
            await window.docx.renderAsync(arrayBuffer, docxContainer, null, {
              className: 'docx',
              inWrapper: true,
              ignoreWidth: false,
              ignoreHeight: false
            });
          } catch (renderErr) {
            console.error('DOCX render error:', renderErr);
            docxContainer.innerHTML = '<div style="text-align:center; padding: 1.5rem; color: var(--text-muted);">In-browser preview not supported for this file structure. Please use the Download button below.</div>';
          }
        }
      }

      setTimeout(() => {
        progressCard.classList.add('hidden');
        successCard.classList.remove('hidden');
      }, 500);

    } catch (err) {
      clearInterval(progressInterval);
      console.error('Conversion error:', err);
      alert(`Conversion Error: ${err.message}\n\nPlease check your backend API URL and ensure pdf_to_docx_service.py is running.`);
      progressCard.classList.add('hidden');
      previewCard.classList.remove('hidden');
    }
  });

  btnStartOver.addEventListener('click', resetUI);
});
