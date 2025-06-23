let accessToken = null;
let restauranteId = null;

// Función auxiliar para mostrar mensajes de error/éxito
function showMessage(message, isSuccess = true) {
    const messageDiv = document.createElement('div');
    messageDiv.textContent = message;
    messageDiv.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 10px 20px;
        background-color: ${isSuccess ? '#2ecc71' : '#e74c3c'};
        color: white;
        border-radius: 5px;
        z-index: 1000;
        animation: fadeIn 0.3s ease-in;
    `;

    document.body.appendChild(messageDiv);

    setTimeout(() => {
        messageDiv.remove();
    }, 3000);
}

// Función para manejar errores de API
async function handleApiError(response) {
    if (!response.ok) {
        const errorData = await response.json().catch(() => ({ message: 'Error desconocido' }));
        const message = errorData.message || errorData.error || 'Error en la operación';
        showMessage(message, false);
        throw new Error(message);
    }
    return response;
}
// Función para asegurar que exista la tabla de resumen
function asegurarTablaResumen() {
    let resumenContainer = document.getElementById('resumen-container');

    if (!resumenContainer) {
        // Crear el contenedor
        resumenContainer = document.createElement('div');
        resumenContainer.id = 'resumen-container';
        resumenContainer.style.marginTop = '20px';

        // Crear título
        const titulo = document.createElement('h3');
        titulo.textContent = 'Resumen de Mesas';
        resumenContainer.appendChild(titulo);

        // Crear tabla
        const tabla = document.createElement('table');
        tabla.id = 'tabla-resumen';
        tabla.border = '1';
        tabla.style.width = '100%';
        tabla.style.maxWidth = '600px';
        tabla.style.margin = '0 auto';

        // Crear encabezado
        const thead = document.createElement('thead');
        thead.innerHTML = `
            <tr>
                <th>#</th>
                <th>Nombre</th>
                <th>Min. Personas</th>
                <th>Max. Personas</th>
                <th>Reservas</th>
            </tr>
        `;
        tabla.appendChild(thead);

        // Crear cuerpo de la tabla
        const tbody = document.createElement('tbody');
        tbody.id = 'resumen-body';
        tabla.appendChild(tbody);

        resumenContainer.appendChild(tabla);

        // Añadir al container del plano
        const planoContainer = document.getElementById('plano-container');
        planoContainer.appendChild(resumenContainer);
    }
}

function actualizarResumen() {
    const resumenBody = document.getElementById('resumen-body');
    if (!resumenBody) return; // no hacer nada si no existe

    const mesas = document.querySelectorAll('.mesa');
    resumenBody.innerHTML = "";

    let contador = 1;
    mesas.forEach(mesa => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
            <td>${contador}</td>
            <td>${mesa.textContent}</td>
            <td>${mesa.dataset.min ?? '-'}</td>
            <td>${mesa.dataset.max ?? '-'}</td>
            <td>0</td>
        `;
        resumenBody.appendChild(tr);
        contador++;
    });
}

// Al final de cargar plano: (versión que crea la tabla si no existe)
async function cargarPlano() {
    try {
        const response = await fetch(`/tables?id_restaurante=${restauranteId}`, {
            headers: {
                'Authorization': `Bearer ${accessToken}`
            }
        });

        await handleApiError(response);
        const data = await response.json();

        mesaCounter = 1;

        // Asegurar que exista la tabla de resumen
        asegurarTablaResumen();

        const resumenBody = document.getElementById('resumen-body');
        resumenBody.innerHTML = "";

        for (const mesa of data) {
            crearMesa(
                mesa.nombre,
                mesa.pos_x,
                mesa.pos_y,
                mesa.size_x,
                mesa.size_y,
                mesa.min_personas,
                mesa.max_personas,
                mesa.forma
            );

            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td>${mesaCounter}</td>
                <td>${mesa.nombre}</td>
                <td>${mesa.min_personas ?? '-'}</td>
                <td>${mesa.max_personas ?? '-'}</td>
                <td>0</td>
            `;
            resumenBody.appendChild(tr);
            mesaCounter++;
        }

        showMessage('Plano cargado correctamente!');
    } catch (error) {
        console.error('Error cargando plano:', error);
        showMessage('Error cargando mesas', false);
    }
}

// Función de login mejorada
async function login() {
    const nombre = document.getElementById('nombre').value;
    const password = document.getElementById('password').value;

    if (!nombre || !password) {
        showMessage('Por favor, complete todos los campos', false);
        return;
    }

    try {
        const response = await fetch('/restaurants/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: nombre, password: password })
        });

        await handleApiError(response);
        const data = await response.json();

        accessToken = data.access_token;
        restauranteId = data.id_restaurante;

        document.getElementById('login-container').style.display = 'none';
        document.getElementById('plano-container').style.display = 'block';

        showMessage(data.message || 'Login correcto!');
        await cargarPlano();
    } catch (error) {
        console.error('Error en login:', error);
    }
}

let mesaCounter = 1;

// Función para crear una nueva mesa visual
function crearMesa(nombre = null, posX = 100, posY = 100, sizeX = 80, sizeY = 80, minPersonas = 2, maxPersonas = 4, forma = "cuadrado") {
    const plano = document.getElementById('plano');

    const mesa = document.createElement('div');
    mesa.classList.add('mesa');
    mesa.style.left = posX + 'px';
    mesa.style.top = posY + 'px';
    mesa.style.width = sizeX + 'px';
    mesa.style.height = sizeY + 'px';
    mesa.dataset.min = minPersonas;
    mesa.dataset.max = maxPersonas;
    mesa.dataset.forma = forma;
    mesa.dataset.numero = mesaCounter;

    if (forma === "circulo") {
        mesa.style.borderRadius = "50%";
    } else {
        mesa.style.borderRadius = "10px";
    }

    mesa.textContent = nombre || `Mesa ${mesaCounter}`;
    mesaCounter++;

    plano.appendChild(mesa);
    actualizarResumen();
    interact(mesa)
        .draggable({
            modifiers: [
                interact.modifiers.restrictRect({
                    restriction: 'parent',
                    endOnly: true
                })
            ],
            listeners: {
                move(event) {
                    const target = event.target;
                    const x = (parseFloat(target.getAttribute('data-x')) || 0) + event.dx;
                    const y = (parseFloat(target.getAttribute('data-y')) || 0) + event.dy;

                    target.style.transform = `translate(${x}px, ${y}px)`;
                    target.setAttribute('data-x', x);
                    target.setAttribute('data-y', y);
                }
            }
        })
        .resizable({
            edges: { left: true, right: true, bottom: true, top: true },
            listeners: {
                move(event) {
                    let { x, y } = event.target.dataset;
                    x = (parseFloat(x) || 0);
                    y = (parseFloat(y) || 0);

                    Object.assign(event.target.style, {
                        width: `${event.rect.width}px`,
                        height: `${event.rect.height}px`,
                    });

                    Object.assign(event.target.dataset, { x: x, y: y });
                }
            }
        });

    // Click derecho para borrar
    mesa.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        if (confirm('¿Quieres eliminar esta mesa?')) {
            mesa.remove();
            showMessage('Mesa eliminada');
            actualizarResumen();
        }
    });

    // Doble click para editar propiedades
    mesa.addEventListener('dblclick', () => {
        editarMesa(mesa);
    });
    actualizarResumen();
}
function organizarMesas() {
    const plano = document.getElementById('plano');
    const mesas = document.querySelectorAll('.mesa');
    if (mesas.length === 0) return;

    const anchoPlano = plano.clientWidth || 800;
    const margen = 20;
    const tamMesa = 80;
    const mesasPorFila = Math.floor((anchoPlano - margen) / (tamMesa + margen));

    mesas.forEach((mesa, index) => {
        const fila = Math.floor(index / mesasPorFila);
        const columna = index % mesasPorFila;

        const posX = margen + columna * (tamMesa + margen);
        const posY = margen + fila * (tamMesa + margen);

        mesa.style.left = posX + 'px';
        mesa.style.top = posY + 'px';
        mesa.style.transform = ''; // resetear posible translate
        mesa.setAttribute('data-x', 0);
        mesa.setAttribute('data-y', 0);
    });

    showMessage('Mesas organizadas automáticamente');
}

// Función para editar mesa mejorada
function editarMesa(mesa) {
    const nuevoNombre = prompt("Nombre de la mesa:", mesa.textContent);
    if (nuevoNombre) {
        mesa.textContent = nuevoNombre;
    }

    const nuevoMin = prompt("Número mínimo de personas:", mesa.dataset.min);
    if (nuevoMin && parseInt(nuevoMin) > 0) {
        mesa.dataset.min = nuevoMin;
    }

    const nuevoMax = prompt("Número máximo de personas:", mesa.dataset.max);
    if (nuevoMax && parseInt(nuevoMax) >= parseInt(mesa.dataset.min)) {
        mesa.dataset.max = nuevoMax;
    }

    const nuevaForma = prompt("Forma (cuadrado o circulo):", mesa.dataset.forma);
    if (nuevaForma && (nuevaForma === "cuadrado" || nuevaForma === "circulo")) {
        mesa.dataset.forma = nuevaForma;
        if (nuevaForma === "circulo") {
            mesa.style.borderRadius = "50%";
        } else {
            mesa.style.borderRadius = "10px";
        }
    }
    actualizarResumen();
}

// Función para guardar el plano mejorada
async function guardarPlano() {
    const mesas = document.querySelectorAll('.mesa');

    if (mesas.length === 0) {
        showMessage('No hay mesas para guardar', false);
        return;
    }

    try {
        // Primero limpiar las mesas existentes
        await fetch(`/tables/clear?id_restaurante=${restauranteId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${accessToken}`
            }
        });

        // Luego guardar las nuevas mesas
        for (const mesa of mesas) {
            const rect = mesa.getBoundingClientRect();
            const plano = document.getElementById('plano').getBoundingClientRect();

            const pos_x = rect.left - plano.left;
            const pos_y = rect.top - plano.top;
            const size_x = rect.width;
            const size_y = rect.height;

            const data = {
                id_restaurante: restauranteId,
                tipo: "mesa",
                nombre: mesa.textContent,
                pos_x: pos_x,
                pos_y: pos_y,
                size_x: size_x,
                size_y: size_y,
                forma: mesa.dataset.forma || "cuadrado",
                reservable: true,
                min_personas: parseInt(mesa.dataset.min) || null,
                max_personas: parseInt(mesa.dataset.max) || null
            };

            const response = await fetch('/tables', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${accessToken}`
                },
                body: JSON.stringify(data)
            });

            await handleApiError(response);
        }

        showMessage('Plano guardado correctamente!');
    } catch (error) {
        console.error('Error guardando plano:', error);
    }
}

function crearNMesas(n) {
    const plano = document.getElementById('plano');
    const anchoPlano = plano.clientWidth || 800;
    const margen = 20;
    const tamMesa = 80;
    const mesasPorFila = Math.floor((anchoPlano - margen) / (tamMesa + margen));

    for (let i = 0; i < n; i++) {
        const fila = Math.floor(i / mesasPorFila);
        const columna = i % mesasPorFila;

        const posX = margen + columna * (tamMesa + margen);
        const posY = margen + fila * (tamMesa + margen);

        crearMesa(null, posX, posY, tamMesa, tamMesa);
    }
}

function crearNMesasPrompt() {
    const n = parseInt(prompt("¿Cuántas mesas quieres añadir?", "10"));
    if (!isNaN(n) && n > 0) {
        crearNMesas(n);
        showMessage(`${n} mesas creadas`);
    }
}
