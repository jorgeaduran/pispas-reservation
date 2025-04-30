let accessToken = null;
let restauranteId = null;
// Al final de cargar plano:
async function cargarPlano() {
    const response = await fetch(`/tables?id_restaurante=${restauranteId}`, {
        headers: {
            'Authorization': `Bearer ${accessToken}`
        }
    });

    if (response.ok) {
        const mesas = await response.json();
        mesaCounter = 1; // Reseteamos

        const resumenBody = document.getElementById('resumen-body');
        resumenBody.innerHTML = ""; // limpiar

        for (const mesa of mesas) {
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

            // Añadir resumen
            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td>${mesaCounter}</td>
                <td>${mesa.nombre}</td>
                <td>${mesa.min_personas ?? '-'}</td>
                <td>${mesa.max_personas ?? '-'}</td>
                <td>0</td> <!-- TODO: en el futuro poner nº de reservas -->
            `;
            resumenBody.appendChild(tr);

            mesaCounter++;
        }

        alert('Plano cargado correctamente!');
    } else {
        alert('Error cargando mesas');
    }
}

// Función de login
async function login() {
    const nombre = document.getElementById('nombre').value;
    const password = document.getElementById('password').value;

    const response = await fetch('/restaurants/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: nombre, password: password })
    });

    if (response.ok) {
        const data = await response.json();
        accessToken = data.access_token;
        restauranteId = data.id_restaurante;

        document.getElementById('login-container').style.display = 'none';
        document.getElementById('plano-container').style.display = 'block';

        alert('Login correcto!');
        await cargarPlano();
    } else {
        alert('Error de login');
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

                    Object.assign(event.target.dataset, {
                        x: x,
                        y: y
                    });
                }
            }
        });

    // Click derecho para borrar
    mesa.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        if (confirm('¿Quieres eliminar esta mesa?')) {
            mesa.remove();
        }
    });

    // Doble click para editar propiedades
    mesa.addEventListener('dblclick', () => {
        const nuevoNombre = prompt("Nombre de la mesa:", mesa.textContent);
        if (nuevoNombre) {
            mesa.textContent = nuevoNombre;
        }

        const nuevoMin = prompt("Número mínimo de personas:", mesa.dataset.min);
        if (nuevoMin) {
            mesa.dataset.min = nuevoMin;
        }

        const nuevoMax = prompt("Número máximo de personas:", mesa.dataset.max);
        if (nuevoMax) {
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
    });
}

// Función para guardar el plano
async function guardarPlano() {
    const mesas = document.querySelectorAll('.mesa');

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
            min_personas: parseInt(mesa.dataset.min) || 2,
            max_personas: parseInt(mesa.dataset.max) || 4
        };

        await fetch('/tables', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${accessToken}`
            },
            body: JSON.stringify(data)
        });
    }

    alert('Plano guardado correctamente!');
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
    }
}