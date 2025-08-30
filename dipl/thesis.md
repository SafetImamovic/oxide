# Uvod

## Problem i motivacija

Sa porastom zahtjeva za interaktivnim 3D sadržajem u web okruženjima (igre, vizualizacije, AR/VR), postoji potreba za lakim, performantnim render engine-om koji može funkcionirati unutar browsera. Trenutna rješenja poput `Three.js` ili `Babylon.js` pružaju kompleksne alate, ali često uključuju višak funkcionalnosti koji opterećuje performanse. Cilj ovog rada je razviti lightweight alternativu optimiziranu za specifične use-caseove.

## Ciljevi rada

- Razvoj modularnog render engine-a baziranog na `WebGPU`, `wgpu`.
- Implementacija osnovnih 3D funkcionalnosti (učitavanje modela, teksture, osvjetljenje).
- Optimizacija za web okruženje (brzo učitavanje, niska memorijska zauzeća).
- Kreiranje jednostavnog API-ja za brzu integraciju.

## Postojeća rješenja

| Rješenje     | Prednosti         | Mane                                  |
|--------------|-------------------|---------------------------------------|
| `Three.js`   | Bogat ecosystem   | Visok overhead za jednostavne scenere |
| `Babylon.js` | Napredne funkcije | Kompleksan API                        |
| `A-Frame`    | Pogodan za AR/VR  | Ograničena fleksibilnost              |

## Metodologija

1. **Analiza zahtjeva**: Definisanje scope-a (šta engine NEĆE raditi).
2. **Dizajn arhitekture**: Odabir WebGPU nad WebGL zbog buduće kompatibilnosti.
3. **Implementacija**: Fokus na:
    - Minimalan rendering pipeline.
    - Batch processing draw callova.
    - LOD (Level of Detail) sistem.
4. **Testiranje**: Metrike FPS-a, memorijske upotrebe i učitavanja u različitim browserima.

# **Kako 3D Modeli Postaju 2D Slika: Od Tačaka do Piksela**


```mermaid
graph LR
A[Web App] --> B[WebGPU API]
B --> C[GPU Driver]
C --> D[RTX 3050]
```



Rendering 3D modela na 2D ekran može zvučati kao magija, ali u osnovi je to samo **niz matematičkih transformacija** koje pretvaraju prostorne podatke u sliku. Umjesto da odmah bacimo sve formule, hajde da krenemo **postupno**—kao što su i grafički sistemi evoluirali—od najjednostavnijih ideja do kompleksnih matrica.

### **Od Kontinuuma do Piksela: Zašto Koristimo Interval [0,1] u Renderiranju?**

Kada pričamo o 3D renderiranju, sve počinje u **kontinuiranom prostoru** $\mathbb{R}^3$, ali ekrani su **diskretni** (sastoje se od konačnog broja piksela). Kako premostiti taj jaz?  
Ključ je u **normalizaciji koordinata u $[0,1]$, što omogućuje jednostavno preslikavanje na bilo koju rezoluciju ekrana. Evo kako to funkcionira:

## Zašto Baš $[0,1]$?

Interval $[0,1]$ je **normalizirani prostor** koji služi kao "most" između:
- **Apsolutnih koordinata** (npr. $(3.2, -1.7, 5.0)$ u 3D svijetu)
- **Relativnih pozicija** unutar neke granice (npr. teksture, prozora projekcije).

#### **Prednosti:**
- **Univerzalnost**: Ne ovisi o stvarnim dimenzijama (radi za bilo koju rezoluciju).
- **Pojednostavljuje interpolaciju** (npr. boje između vrhova).
- **Olakšava GPU optimizacije** (hardver voli raditi s normaliziranim vrijednostima).


## Kako Dolazimo do $[0,1]$?
### **Korak 1: Projektujemo 3D tačke u "clip space" (homogene koordinate)**
Nakon množenja sa **projekcionom matricom**, dobijamo koordinate u tzv. *clip space*-u:  
$$
\mathbf{v}_{\text{clip}} = \begin{bmatrix} x \\ y \\ z \\ w \end{bmatrix}
$$
- Ovdje $x, y, z$ mogu biti bilo koji realni brojevi: $x, y, z \in \mathbb{R}$.

### **Korak 2: Perspektivna podjela (normalizacija u NDC)**
Da bismo dobili **normalizirane koordinate (NDC)**, dijelimo sa $w$:  
$$
\mathbf{v}_{\text{NDC}} = \begin{bmatrix} x/w \\ y/w \\ z/w \end{bmatrix}
$$
- **NDC prostor** je kocka $[-1, 1]^3$ (za OpenGL) ili $[0,1]^3$ (za Vulkan/DirectX/**WebGPU**).

### **Korak 3: Preslikavanje u $[0,1]$ za viewport**
Za ekranske koordinate, NDC se transformiše u $[0,1]$ x $[0,1]$:  
$$
\begin{cases}
x_{\text{norm}} = \frac{x_{\text{NDC}} + 1}{2} \\
y_{\text{norm}} = \frac{y_{\text{NDC}} + 1}{2}
\end{cases}
$$
- Sada su sve vrijednosti u intervalu **[0,1]**, nezavisno od rezolucije ekrana.

## Diskretizacija: Od $[0,1]$ do Piksela
Konačni korak je pretvorba u **cijele brojeve** (piksele):  
$$
\begin{cases}
x_{\text{pixel}} = \lfloor x_{\text{norm}} \cdot \text{width} \rfloor \\
y_{\text{pixel}} = \lfloor (1 - y_{\text{norm}}) \cdot \text{height} \rfloor \quad \text{(Y-os je obrnuta!)}
\end{cases}
$$
#### **Primjer:**
Za ekran 1920x1080 i tačku $(0.5, 0.2)$ u $[0,1]$:
- $x_{\text{pixel}} = 0.5 \times 1920 = 960$
- $y_{\text{pixel}} = (1 - 0.2) \times 1080 = 864$

## Zašto Ovo Radi u Praksi?

- **Fleksibilnost**: Ako promijenimo rezoluciju ekrana, [0,1] se automatski skalira.
- **Efikasnost**: GPU brzo radi s normaliziranim vrijednostima prije nego što ih pretvori u piksele.
- **Interpolacija**: Svi parametri (boje, UV koordinate) se interpoliraju u [0,1] prije rasterizacije.

## Vizuelna Analogija

Zamislite da imate:
1. **Gumeni list papira** (kontinuirani prostor $\mathbb{R}^2$).
2. **Mrežu piksela** (ekran).
3. **Šablon $[0,1]$ x $[0,1]$ koji rastežete preko mreže.

Normalizacija u [0,1] je kao **precizno nanošenje šablona** na mrežu, bez obzira na njenu veličinu!

### **Zaključak**
Interval $[0,1]$ je **"zlatna sredina"** između:
- Matematičke kontinualnosti $\mathbb{R}$
- Digitalne diskretnosti (pikseli).

Bez njega, render motori ne bi mogli **adaptirati se na različite rezolucije** tako elegantno.

## Početak: Definicija Tačaka u 2D Prostoru

Definisati ćemo skup koji će služiti kao

Prije nego što uopće razmišljamo o 3D svijetu, počnimo s nečim što nam je poznato—**ravan crtanja** (2D prostor).


\begin{figure}
\centering
\begin{tikzpicture}[scale=1,>=stealth]

% Draw axes
\draw[->] (-4,0) -- (4,0) node[right] {$x$};
\draw[->] (0,-4) -- (0,4) node[above] {$y$};

% Draw grid (optional)
\draw[step=1cm,gray,very thin] (-4,-4) grid (4,4);

% Tick marks on x-axis
\foreach \x in {-3,-2,-1,1,2,3}
\draw (\x,0.1) -- (\x,-0.1) node[below] {\x};

% Tick marks on y-axis
\foreach \y in {-3,-2,-1,1,2,3}
\draw (0.1,\y) -- (-0.1,\y) node[left] {\y};

\end{tikzpicture}
\caption{Your caption here}
\label{fig:yourlabel1}
\end{figure}



\begin{figure}[h!]
\centering
\begin{tabular}{m{0.24\textwidth} m{0.6\textwidth}}
$\displaystyle
\begin{aligned}
\mathbf{a} &= \langle 1, 3 \rangle   & = \begin{bmatrix} 1 \\ 3 \end{bmatrix} \\
\mathbf{b} &= \langle 3, -1 \rangle  & = \begin{bmatrix} 3 \\ -1 \end{bmatrix} \\
\mathbf{c} &= \langle -3, -3 \rangle & = \begin{bmatrix} -3 \\ -3 \end{bmatrix}
\end{aligned}
$
&
\begin{tikzpicture}[scale=1]
% Draw axes
\draw[latex-latex] (-4,0) -- (4,0) node[right] {$x$};
\draw[latex-latex] (0,-4) -- (0,4) node[above] {$y$};

% Draw grid (optional)
\draw[step=1cm,gray,very thin] (-4,-4) grid (4,4);

% Tick marks on x-axis
\foreach \x in {-3,-2,-1,1,2,3}
\draw (\x,0.1) -- (\x,-0.1) node[below] {\x};

% Tick marks on y-axis
\foreach \y in {-3,-2,-1,1,2,3}
\draw (0.1,\y) -- (-0.1,\y) node[left] {\y};

% \fill (1,3) circle (2pt); \node[right] at (1,3.25) {$\mathbf{a}$};
% \fill (3,-1) circle (2pt); \node[right] at (3,-1.25) {$\mathbf{b}$};
% \fill (-3,-3) circle (2pt); \node[right] at (-3.5,-3.25) {$\mathbf{c}$};

% Draw vectors with bigger arrowheads
\draw[big arrow] (0,0) -- (1,3) ; \node[right] at (1,3.25) {$\mathbf{a}$};
\draw[big arrow] (0,0) -- (3,-1) ; \node[right] at (3,-1.25) {$\mathbf{b}$};
\draw[big arrow] (0,0) -- (-3,-3) ; \node[right] at (-3.5,-3.25) {$\mathbf{c}$};

% Clockwise rotation arrow near top-right corner
\draw[big arrow, ultra thick]
(-0.75,0.75)
arc[start angle=135, end angle=-135, radius=1cm];
\end{tikzpicture}
\end{tabular} \\
\begin{tikzpicture}[scale=1]
% Draw axes
\draw[latex-latex] (-4,0) -- (4,0) node[right] {$x$};
\draw[latex-latex] (0,-4) -- (0,4) node[above] {$y$};

% Draw grid (optional)
\draw[step=1cm,gray,very thin] (-4,-4) grid (4,4);

% Tick marks on x-axis
\foreach \x in {-3,-2,-1,1,2,3}
\draw (\x,0.1) -- (\x,-0.1) node[below] {\x};

% Tick marks on y-axis
\foreach \y in {-3,-2,-1,1,2,3}
\draw (0.1,\y) -- (-0.1,\y) node[left] {\y};

\fill (1,3) circle (2pt); \node[right] at (1,3.25) {$\mathbf{a}$};
\fill (3,-1) circle (2pt); \node[right] at (3,-1.25) {$\mathbf{b}$};
\fill (-3,-3) circle (2pt); \node[right] at (-3.5,-3.25) {$\mathbf{c}$};

% Draw vectors with bigger arrowheads
\draw[big arrow] (0,0) -- (1,3) ; \node[right] at (1,3.25) {$\mathbf{a}$};
\draw[big arrow] (0,0) -- (3,-1) ; \node[right] at (3,-1.25) {$\mathbf{b}$};
\draw[big arrow] (0,0) -- (-3,-3) ; \node[right] at (-3.5,-3.25) {$\mathbf{c}$};

\draw[big arrow] (1,3) -- (3,-1) ; \node[right] at (1,3.25) {$\mathbf{a}$};
\draw[big arrow] (3,-1) -- (-3,-3) ; \node[right] at (3,-1.25) {$\mathbf{b}$};
\draw[big arrow] (-3,-3) -- (1,3) ; \node[right] at (-3.5,-3.25) {$\mathbf{c}$};

% Clockwise rotation arrow near top-right corner
\draw[big arrow, ultra thick]
(-0.75,0.75)
arc[start angle=135, end angle=-135, radius=1cm];
\end{tikzpicture}

\caption{Definicija Trougla Putem Vektora.}
\label{fig:yourlabel}
\end{figure}






- **Tačka u 2D**: Možemo je predstaviti uređenim parom $\langle x, y \rangle$.
- **Linija između dvije tačke**: Ako imamo $\mathbf{v}_0 = (x_0, y_0)$ i $\mathbf{v}_1 = (x_1, y_1)$, linija se može opisati jednadžbom:  
  $$
  y = y_0 + t(y_1 - y_0), \quad t \in [0, 1]
  $$


Ovo je dovoljno za crtanje osnovnih oblika, ali šta ako želimo **rotirati** ili **povećati** neki objekat?

## **2. Transformacije u 2D: Pomjeranje, Rotacija, Skaliranje**
### **(a) Translacija (Pomjeranje)**
Da bismo pomjerili tačku $(x, y)$ za $(t_x, t_y)$, jednostavno **dodajemo pomak**:  
$$
(x', y') = (x + t_x, y + t_y)
$$  
Ali šta ako želimo **kombinovati** više transformacija?

### **(b) Rotacija**
Rotacija oko ishodišta za ugao $\alpha$ može se opisati **matricom**:  
$$
\begin{bmatrix} x' \\ y' \end{bmatrix} = \begin{bmatrix} \cos\alpha & -\sin\alpha \\ \sin\alpha & \cos\alpha \end{bmatrix} \begin{bmatrix} x \\ y \end{bmatrix}
$$

### **(c) Skaliranje (Povećanje/Smanjenje)**
Množenjem koordinata faktorima $(s_x, s_y)$:  
$$
(x', y') = (x \cdot s_x, y \cdot s_y)
$$

### **Problem: Kako kombinovati ove operacije?**
Da bismo **rotirali** i **pomjerili** objekat, moramo prvo rotirati, pa dodati translaciju. Ali ovo postaje nezgrapno kad imamo više koraka.

**Rješenje? Matrične transformacije u homogenim koordinatama!**

### **Homogene koordinate i afine transformacije: Zašto su ključne u 3D renderiranju?**

Kada radimo s 3D grafikom, želimo **jednostavno kombinovati** pomjeranje, rotaciju i skaliranje. Obične 3D koordinate $\langle x, y, z \rangle$ nisu dovoljne za to, jer:
- **Translacija nije linearna operacija** (ne može se izraziti samo množenjem matrice i vektora).
- **Kombinovanje rotacija i translacija zahtijeva nezgrapno ulančavanje operacija**.

Rješenje? **Homogene koordinate!**

## **1. Šta su homogene koordinate?**
To je proširenje standardnih koordinata s dodatnom **četvrtom komponentom** $w$:
- **Tačka u 3D prostoru**: $\langle x, y, z, 1 \rangle$
- **Vektor (smjer)**: $\langle x, y, z, 0 \rangle$

#### **Zašto $w = 1$ za tačke, a $w = 0$ za vektore?**
- **Tačke** se pomjeraju pri translaciji.
- **Vektori** (npr. normale) ne mijenjaju se pri pomjeranju.

Primjer:
- Translacija tačke $(2, 3, 5, 1)$ za $(1, 0, 0)$ daje $(3, 3, 5, 1)$.
- Translacija vektora $(0, 1, 0, 0)$ ostaje $(0, 1, 0, 0)$ (smjer se ne mijenja).

## **2. Afine transformacije u homogenim koordinatama**
Afina transformacija je svaka operacija koja **čuva pravolinijske odnose** (npr. rotacija, skaliranje, translacija). U homogenim koordinatama, sve se ove operacije mogu izraziti **množenjem matrice i vektora**.

### **(a) Translacija (Pomjeranje)**
Matrica translacije u 3D:  
$$
T = \begin{bmatrix}
1 & 0 & 0 & t_x \\
0 & 1 & 0 & t_y \\
0 & 0 & 1 & t_z \\
0 & 0 & 0 & 1
\end{bmatrix}, \quad \mathbf{v}' = T \cdot \mathbf{v}
$$  
Primjer: Pomjeranje tačke \((1, 2, 3, 1)\) za \((4, 5, 6)\):  
$$
\begin{bmatrix}
1 & 0 & 0 & 4 \\
0 & 1 & 0 & 5 \\
0 & 0 & 1 & 6 \\
0 & 0 & 0 & 1
\end{bmatrix}
\begin{bmatrix}
1 \\ 2 \\ 3 \\ 1
\end{bmatrix}
=
\begin{bmatrix}
1+4 \\ 2+5 \\ 3+6 \\ 1
\end{bmatrix}
=
\begin{bmatrix}
5 \\ 7 \\ 9 \\ 1
\end{bmatrix}
$$

### **(b) Rotacija**
Rotacija oko **X-ose** za ugao $\alpha$:  
$$
R_x = \begin{bmatrix}
1 & 0 & 0 & 0 \\
0 & \cos\alpha & -\sin\alpha & 0 \\
0 & \sin\alpha & \cos\alpha & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}
$$  
(Analogno za $R_y$ i $R_z$.)

### **(c) Skaliranje**
$$
S = \begin{bmatrix}
s_x & 0 & 0 & 0 \\
0 & s_y & 0 & 0 \\
0 & 0 & s_z & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}
$$  
Primjer: Skaliranje za $\langle 2, 1, 0.5 \rangle$:  
$$
\begin{bmatrix}
2 & 0 & 0 & 0 \\
0 & 1 & 0 & 0 \\
0 & 0 & 0.5 & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}
\begin{bmatrix}
1 \\ 1 \\ 1 \\ 1
\end{bmatrix}
=
\begin{bmatrix}
2 \\ 1 \\ 0.5 \\ 1
\end{bmatrix}
$$

## **3. Prednosti homogenih koordinata**
1. **Sve transformacije su matrično množenje** (što GPU brzo izračunava).
2. **Lako kombinovanje operacija**:  
   $$
   \mathbf{M} = T \cdot R \cdot S \quad \text{(jedna matrica umjesto više koraka)}
   $$
3. **Perspektivna projekcija** može se izraziti matricom (ključno za 3D renderiranje).

## **4. Kako homogene koordinate omogućavaju perspektivu?**
Perspektivna projekcija **smanjuje udaljene objekte**. Ovo se postiže dijeljenjem sa $w$ nakon transformacije:  
$$
\mathbf{v}_{\text{clip}} = \begin{bmatrix} x \\ y \\ z \\ w \end{bmatrix} \implies \mathbf{v}_{\text{2D}} = \begin{bmatrix} x/w \\ y/w \end{bmatrix}
$$
- Za ortografske projekcije, $w = 1$.
- Za perspektivne, $w$ ovisi o dubini (npr. $w = z$).

Primjer:  
$$
\begin{bmatrix}
1 \\ 2 \\ 3 \\ 2
\end{bmatrix}
\implies
\begin{bmatrix}
0.5 \\ 1 \\ 1.5
\end{bmatrix}
$$

## **5. Kada homogene koordinate nisu potrebne?**
- **2D crtanje bez translacija** (npr. skaliranje/rotacija oko ishodišta).
- **Fizičke simulacije** gdje se koriste samo vektori (npr. sile).

Ali u 3D renderiranju, **homogene koordinate su neizostavne** jer omogućavaju:
- Jedinstvenu reprezentaciju tačaka i vektora.
- Efikasno kombinovanje svih transformacija.
- Perspektivno renderiranje.


## **3. Prelazak na 3D i Uvođenje Homogenih Koordinata**
U 3D prostoru, tačke imaju tri koordinate $(x, y, z)$. Ali kako bismo **elegantno** kombinovali rotacije, translacije i skaliranje, koristimo **homogene koordinate**—dodajemo četvrtu komponentu $w$ (obično $1$ za tačke, $0$ za vektore).

- **Tačka u homogenim koordinatama**: $(x, y, z, 1)$
- **Vektor (smjer)**: $(x, y, z, 0)$

**Zašto ovo koristimo?**
- **Translacija postaje množenje matrice** (umjesto zasebnog sabiranja).
- Sve transformacije se mogu **kombinovati u jednu matricu**.


## **4. 3D Transformacije: Model, Pogled, Projekcija**
### **(a) Model Matrix $\mathbf{M}$**
Definiše **položaj, rotaciju i veličinu** objekta u svijetu.
- **Primjer**: Ako želimo rotirati kocku za 45° (oko neke ose) i pomjeriti je:  
  $$
  \mathbf{M} = T \cdot R \quad \text{(prvo rotacija, pa translacija)}
  $$

### **(b) View Matrix $\mathbf{V}$**

Odgovara na pitanje: **"Kako kamera vidi scenu?"**
- Ako se kamera pomjeri, svi objekti se **transformišu u suprotnom smjeru**.

### **(c) Projection Matrix $\mathbf{P}$**
Pretvara 3D koordinate u **2D prostor ekrana** koristeći perspektivu.
- **Perspektivna projekcija** čini da se udaljeni objekti smanjuju.

Konačna transformacija:  
$$
\mathbf{v}_{\text{ekran}} = \mathbf{P} \cdot \mathbf{V} \cdot \mathbf{M} \cdot \mathbf{v}
$$

## **5. Rasterizacija: Od Trouglova do Piksela**
Kada imamo 2D koordinate, **GPU pretvara trouglove u piksele**:
1. **Interpolacija boja** između vrhova.
2. **Z-buffering** (test dubine) osigurava da se bliži objekti crtaju iznad udaljenih.


## **Zaključak: Zašto Je Sve Ovo Važno?**
Rendering pipeline nije samo suha matematika—to je **način na koji računar vidi svijet**. Kroz transformacije, projekciju i rasterizaciju, možemo simulirati **prostor, svjetlost i materijale**, pretvarajući apstraktne podatke u sliku koju ljudi mogu razumjeti.

Ako želite da detaljnije istražimo bilo koji korak (npr. kako perspektivna podjela radi ili kako se optimizuje brzina renderovanja), slobodno pitajte!



# Example Rust Code

```rust
fn main() {
    println!("Hello World!"); 
}
```
\captionof{code}{"Hello World!" u Rust-u}
\label{code:code1}

# Example Latex
Perspective Projection Matrix:

$$
\begin{bmatrix}
\frac{1}{\tan(\theta/2) \cdot a} & 0 & 0 & 0 \\
0 & \frac{1}{\tan(\theta/2)} & 0 & 0 \\
0 & 0 & -\frac{f + n}{f - n} & -\frac{2fn}{f - n} \\
0 & 0 & -1 & 0
\end{bmatrix}
$$
Example Usage:
$$
\begin{bmatrix}
\frac{1}{1 \cdot \frac{16}{9}} & 0 & 0 & 0 \\
0 & \frac{1}{1} & 0 & 0 \\
0 & 0 & -\frac{100.0 + 0.1}{100.0 - 0.1} & -\frac{2 \cdot 100.0 \cdot 0.1}{100.0 - 0.1} \\
0 & 0 & -1 & 0
\end{bmatrix}
=
\begin{bmatrix}
\frac{9}{16} & 0 & 0 & 0 \\
0 & 1 & 0 & 0 \\
0 & 0 & -\frac{100.1}{99.9} & -\frac{20.0}{99.9} \\
0 & 0 & -1 & 0
\end{bmatrix}
$$


$$
\int_0^1 x^2 \, dx = \left[ \frac{x^3}{3} \right]_0^1 = \frac{1}{3}
$$



### **Jednačina Renderinga (Kajiyina Formulacija)**

$$
L_o(\mathbf{x}, \omega_o, \lambda, t) = L_e(\mathbf{x}, \omega_o, \lambda, t) + \int_{\Omega} f_r(\mathbf{x}, \omega_i, \omega_o, \lambda, t) \, L_i(\mathbf{x}, \omega_i, \lambda, t) \, (\omega_i \cdot \mathbf{n}) \, d\omega_i
$$
- $L_o$: Radijancija koja napušta tačku $\mathbf{x}$ u pravcu $\omega_o$.
- $L_e$: Emitirana radijancija (izvori svjetlosti).
- $f_r$: BRDF (Bidirekciona funkcija distribucije refleksije).
- $L_i$: Dolazna radijancija iz pravca $\omega_i$.
- $\omega_i \cdot \mathbf{n}$: Kosinusni član za ugao površine.
- $\lambda, t$: Talasna dužina i vrijeme (za spektralni/privremeni rendering).


### **Monte Karlo Estimator (Praktična Implementacija)**
$$
\hat{L}_o \approx \frac{1}{N} \sum_{k=1}^N \frac{f_r(\omega_{i,k}, \omega_o) \, L_i(\omega_{i,k}) \, (\omega_{i,k} \cdot \mathbf{n})}{p(\omega_{i,k})}
$$
- $N$: Broj uzoraka zraka.
- $p(\omega_{i,k})$: Funkcija gustine vjerovatnoće (PDF) za uzorkovanje po važnosti.



### **Kombinacija sa Participirajućim Medijem (Volumetrijski Rendering)**
$$
L_o = \int_{t=0}^d \exp\left(-\int_{s=0}^t \sigma_t(\mathbf{x}(s)) \, ds\right) \sigma_s(\mathbf{x}(t)) \int_{\Omega} f_p(\omega_i, \omega_o) L_i(\omega_i) \, d\omega_i \, dt
$$
- $\sigma_t, \sigma_s$: Koeficijenti ekstinkcije i raspršenja.
- $f_p$: Fazna funkcija (za raspršenje).

### **Bonus: GPU-Optimizirana Aproksimacija (Pojednostavljena za WebGL)**
$$
L_o \approx \text{envMap}(\omega_o) + \sum_{k=1}^3 \frac{\text{albedo}}{\pi} \frac{I_k \, (\mathbf{n} \cdot \omega_{i,k})}{\|\mathbf{x} - \mathbf{x}_k\|^2}
$$
- $\text{envMap}$: Unaprijed izračunato osvjetljenje okoline.
- $I_k$: Intenzitet \(k\)-te tačkaste svjetlosti.



# Razvoj Lightweight 3D Render Engine-a za Web Aplikacije

## Cilj Rada, Target
### Initial Target

Cilj rada je razviti jednostavan, optimizovan **3D render engine** koji radi u **browseru** koristeći **WebGL** ili **WebGPU** koji bi trebao omogućiti **učitavanje i prikaz 3D modela, osvjetljenje, teksture i osnovne efekte**, uz fokus na **performanse i optimizaciju za web okruženje**.

### Refined Target

Cilj ovog rada je razvoj **laganog i optimizovanog 3D render engine-a** koji radi u **browseru** i **nativno** koristeći **WebGPU** i **Rust**. Engine će omogućiti:

1. **Učitavanje 3D modela** iz standardnih formata (glTF*, **OBJ**)
2. **Prikaz 3D modela** sa podrškom za osnovne **geometrijske transformacije**
3. **Osvjetljenje** (osnovni PBR model ili Phong osvetljenje)
4. **Podršku za teksture** i njihovo mapiranje
5. **Implementaciju osnovnih vizuelnih efekata**

Poseban fokus biće na **performansama i optimizaciji**, uključujući:

- **Efikasno korišćenje WebGPU API-ja** za paralelizaciju i ubrzanje renderinga, koristenje compute shaders.
- **Optimizaciju memorije** i **minimizaciju CPU-GPU overhead-a**
- **Upotrebu Rust-a** za bezbjedno i performantno upravljanje resursima

Ovaj render engine će omogućiti istraživanje **modernih tehnika renderovanja u web okruženju**, sa potencijalnom primjenom u **igrama, interaktivnim simulacijama i vizuelizacijama**.

## Kljucne Rijeci, Key Words, Terminologija

- **Lightweight (Lagan)**
- **3D Render Engine**
- **Za Web Aplikacije (Browser Support)**
- **WebGL**
- **WebGPU**
- **Ucitavanje 3D Modela**
- **Prikaz 3D Modela**
- **Osvjetljenje (Lighting)**
- **Teksture (Textures)**
- **Osnovni Efekti (Basic Effects)**
- **Performance (Performanse)**
- **Optimizacija (Optimization)**

## Plan Istrazivanja, Research Plan

# Plan učenja i izvori za razvoj lightweight 3D render engine-a u WebGPU i Rust-u

Ovaj plan pokriva **matematiku, programiranje, WebGPU i optimizaciju** – sve što treba za razvoj render engine-a za browser.

## Plan učenja

### 1. Matematika za 3D renderovanje

- **Knjiga**: [[Mathematics for 3D Game Programming and Computer Graphics (Third Edition).pdf]] – glavni izvor
- **Online**:
    - Linearna algebra: [Interactive Linear Algebra](https://textbooks.math.gatech.edu/ila/)
    - Grafičke transformacije: [Scratchapixel](https://www.scratchapixel.com/)

**Cilj**: Razumjeti linearne transformacije, matrice, kvaternione, osvetljenje i interpolaciju.

### 2. Osnove renderovanja i GPU arhitekture

- **Knjiga**: [[Real-Time Rendering.3rd.pdf]] (4th ed.) – temeljni koncepti renderovanja
- **Online**:
    - [Scratchapixel](https://www.scratchapixel.com/) – teorija renderovanja
    - [WebGPU fundamentals](https://webgpufundamentals.org/) – moderni GPU koncepti

**Cilj**: Razumjeti pipeline (vertex shader, rasterizacija, fragment shader), PBR osvetljenje i optimizaciju GPU koda.

### 3. Rust i programski jezik

- **Knjiga**: _The Rust Book_ ([https://doc.rust-lang.org/book/](https://doc.rust-lang.org/book/)) – osnova jezika
- **Online**:
    - [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
    - [Rust WebGPU tutorials](https://sotrh.github.io/learn-wgpu/)

**Cilj**: Naučiti Rust i razumjeti kako upravljati memorijom i sistemskim resursima.

### 4. WebGPU i API renderovanja

- **Online**:
    - [WebGPU Fundamentals](https://webgpufundamentals.org/) – osnova API-ja
    - [Learn WGPU](https://sotrh.github.io/learn-wgpu/) – implementacija render pipeline-a u Rust-u
    - [wgpu-rs GitHub repo](https://github.com/gfx-rs/wgpu)

**Cilj**: Razumjeti kako WebGPU radi i kako ga koristiti za renderovanje scena.

### 5. Optimizacija i implementacija 3D engine-a

- **Knjiga**: [[Game Engine Architecture.pdf]] – kako su dizajnirani veliki engine-i
- **Online**:
    - Optimizacija WebGPU: [WebGPU Performance Best Practices](https://webgpufundamentals.org/webgpu-performance.html)
    - GPU optimizacije: [GPU Gems](https://developer.nvidia.com/gpugems/gpugems)

**Cilj**: Naučiti kako optimizovati WebGPU aplikacije za performanse.

## Plan rada (4-6 meseci)

### Mjesec 1-2: Osnove matematike, Rust-a i render pipeline-a

Naučiti linearnu algebru, matrice, transformacije  
Naučiti Rust (The Rust Book)  
Razumjeti GPU arhitekturu i render pipeline

### Mjesec 3: WebGPU i prvi renderer

Početi implementaciju sa WebGPU i Rust-om  
Napraviti osnovni pipeline za učitavanje i prikaz modela

### Mjesec 4-5: Osvetljenje, teksture, optimizacija

Dodati Phong/PBR osvetljenje  
Implementirati podršku za teksture  
Optimizovati render pipeline

### Mjesec 6: Dovršavanje i dokumentacija

Optimizacija i debugging  
Pisanje dokumentacije i testiranje

# Jezik & Tehnoloski Stack

## **Uvod: Odabir Jezika i Tehnološkog Stack-a**

Prilikom razvoja **lightweight 3D render engine-a za web aplikacije**, ključno je odabrati programski jezik i tehnologije koje omogućavaju **visoke performanse, sigurnost i optimalno korišćenje hardvera**. Postoje tri glavne opcije za implementaciju:

1. **JavaScript** – dominantan jezik za web aplikacije, ali nije optimalan za performanse.
2. **C/C++** – tradicionalno korišćeni jezici za grafiku i game engine-e, ali pate od problema sa bezbjednošću memorije.
3. **Rust** – moderan, **memory-safe** jezik koji kombinuje visoke performanse C/C++-a sa bezbjednošću upravljanja memorijom.

Na osnovu ovih faktora, **Rust** je izabran kao primarni jezik za implementaciju **3D render engine-a** zbog svoje **sigurnosti, efikasnosti i nativne podrške za WebGPU** (Pomocu `wgpu`).

## **Poređenje JavaScript-a, C/C++-a i Rust-a**

### **1. JavaScript vs. C/C++/Rust**

JavaScript je najčešće korišćeni jezik za web aplikacije, ali **nije dizajniran za high-performance 3D rendering**.

**WebIDL (Web Interface Definition Language)** je jezik koji se koristi za definisanje interfejsa koje veb pretraživači implementiraju – u suštini, to je način na koji pretraživač izlaže izvornu funkcionalnost (kao što su **WebGPU**, **WebGL** ili **DOM API-ji**) JavaScript-u.

\begin{quote}
\textbf{Abstract}
\textit{This standard defines an interface definition language, Web IDL, that can be used to describe interfaces that are intended to be implemented in web browsers.}
\end{quote}
\textit{Izvor:} \url{https://webidl.spec.whatwg.org}

| **Osobina**                  | **JavaScript**               | **C/C++**                    | **Rust**                     |
|------------------------------|------------------------------|------------------------------|------------------------------|
| **Brzina**                   | Sporiji<br>(JIT kompilacija) | Nativni kod                  | Nativni kod                  |
| **Upravljanje<br>memorijom** | Automatski (GC)              | Ručno (moguć<br>memory leak) | Sigurno <br>(borrow checker) |
| **Podrška za WebGPU**        | Kroz API                     | Nativno                      | Nativno (wgpu-rs)            |
| **Multi-threading**          | Ograničen<br>(Web Workers)   | Pravi thread-ovi             | Pravi thread-ovi             |



\begin{tabular}{|p{3cm}|p{4cm}|p{4cm}|p{4cm}|}
\hline
\textbf{Osobina} & \textbf{JavaScript} & \textbf{C/C++} & \textbf{Rust} \\
\hline
Brzina & Sporiji \\ (JIT) & Nativni kod & Nativni kod \\
\hline
Upravljanje memorijom & Automatski (GC) & Ručno (memory leak) & Sigurno (borrow checker) \\
\hline
Podrška za WebGPU & Kroz API & Nativno & Nativno (wgpu-rs) \\
\hline
Multi-threading & Ograničen (Web Workers) & Pravi thread-ovi & Pravi thread-ovi \\
\hline
\end{tabular}


**Zaključak:** JavaScript nije idealan za 3D rendering jer je sporiji i nema pravu kontrolu nad memorijom.

### **2. C/C++ vs. Rust**

**C i C++** su dugo dominirali u domenu **grafičkog programiranja i game engine-a**, ali imaju **kritične probleme sa upravljanjem memorijom**.

**CISA (Cybersecurity and Infrastructure Security Agency) izveštaj ("CISA Report Finds Most Open-Source Projects Contain Memory-Unsafe Code")** pokazuje da je **ogroman procenat sigurnosnih problema** u softveru posljedica jezika **bez sigurnog upravljanja memorijom**, poput **C i C++**.

**The Case for Memory Safe Roadmaps** ističe jezike koji pružaju **sigurnost memorije**, uključujući:  
**Rust, C#, Go, Java, Python, Swift** – od kojih je **Rust najbrži**.

#### **Glavne razlike između C/C++ i Rust-a:**

| **Osobina**                        | **C/C++**                                                  | **Rust**                            |
|------------------------------------|------------------------------------------------------------|-------------------------------------|
| **Sigurnost memorije**             | Ne postoji <br>(mogući buffer overflow,<br>use-after-free) | Borrow checker <br>sprječava greške |
| **Ručno upravljanje<br>memorijom** | Ali sa velikim rizicima                                    | Bezbjedno i eksplicitno             |
| **Concurrency**                    | Podložan race condition-ima                                | Sigurna paralelizacija              |
| **Biblioteke**                     | Vulkan, OpenGL                                             | wgpu-rs, Bevy, gfx-rs               |

**Zaključak:** Rust pruža **performanse C/C++-a** uz **sigurnost memorije**, čime eliminiše čitavu klasu bagova koji su često uzrok pada sistema.

### **Zašto Rust za 3D Render Engine?**

**Visoke performanse** – Rust se kompajlira u **nativni kod** i koristi **SIMD (Single Instruction Multiple Data) i multi-threading**.  
**Bezbjedno upravljanje memorijom** – nema **buffer overflow-a** i **use-after-free** problema.  
**Direktna podrška za WebGPU** – **wgpu-rs** omogućava nativno korišćenje WebGPU-a.  
**Ekosistem** – Rust ima moderne grafičke biblioteke (wgpu, Bevy, gfx-rs).  
**Bolja optimizacija za Web** – Rust kod može biti kompajliran u **WASM** (WebAssembly), omogućavajući efikasno pokretanje u browseru.

**Zaključak:** Rust je trenutno **najbolji izbor** za razvoj **sigurnog, brzog i optimizovanog 3D render engine-a za web aplikacije**.
## Definicija Render Engine-a

### Šta znači **"engine"** u softveru?

U softverskom kontekstu, **engine** (ili **motor**) je **osnovna komponenta ili sistem** koji je napravljen da obavlja **određeni skup zadataka**, često na način koji je ponovo upotrebljiv i modularan. Može se zamisliti kao motor automobila — on pokreće određene funkcije, ali nije cijeli automobil.

U softveru, engine obično:

- Prima ulazne podatke (npr. 3D modeli, svjetla, teksture),
- Obrađuje ih prema pravilima,
- I daje izlaz (npr. sliku, animaciju, simulaciju itd.).

#### Primjeri:

- **Game engine** (eng. za “igrački motor”, "motor za video igre") upravlja fizikom, grafikom, unosom korisnika i drugim aspektima igre.
- **Physics engine** (eng. za "motor za fiziku") simulira zakone fizike poput gravitacije, sudara itd.
- **Render engine** je zadužen za “pretvaranje” 3D scene u 2D sliku.

### Šta je **render engine**?

**Render engine** (rendererski motor) je softver koji **pretvara 3D podatke o sceni u 2D sliku ili animaciju**, često sa ciljem da izgleda realistično (ili stilizovano). On simulira ponašanje svjetla, materijala, kamera i drugih vizuelnih efekata da bi napravio konačnu sliku.

#### Ključne funkcije render engine-a:

- Prikaz svjetla i sjena
- Izračun refleksije, refrakcije (lom svjetlosti), ambijentalne okluzije
- Primjena materijala i tekstura
- Simulacija interakcije svjetla sa površinama (poznato kao fizički bazirano renderovanje)

### Primjeri render engine-a:

- **Cycles** (u Blenderu): fizički bazirani renderer
- **Eevee** (takođe u Blenderu): renderer za realno-vrijeme
- **V-Ray**, **Octane**, **Arnold**: profesionalni render engine-i za filmove, arhitekturu itd.
- **WebKit / Blink**: u web preglednicima, ovo su takođe "rendering engine-i" jer pretvaraju HTML i CSS u ono što vidiš na ekranu (druga vrsta renderovanja)

### Ukratko:

- **Engine** = sistem koji obavlja određeni tip zadataka (grafika, fizika, itd.)
- **Render engine** = softver koji iz 3D podataka pravi 2D sliku


\listoffigures    
\listofcodes      
\listofdiagrams      
\listoftables    