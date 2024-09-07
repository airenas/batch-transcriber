# Diegimas naudojant *Docker*

Dipolis projektui

## Apie

`Batch transcriber` sistema yra realizuota *Docker* komponentais. Visa sistema sukonfigūruota ir paruošta paleisti su *docker compose* konfigūraciniu failu.


## Reikalavimai

### Aparatūrai

| Komponen-tas | Min reikalavimai | Rekomenduo-jama | Papildomai |
| -----------------|------------------|---------------------|-------------------------------------------|
| Platform | x86_64 | | |
| CPU | 64-bit, 2 branduoliai | 4 branduoliai | |
| HDD | 10 Gb | 15Gb  | |
| RAM | 4 Gb | 8 Gb | |

### Programinei įrangai

#### OS

Linux OS 64-bit (papildomai žiūrėkite [reikalavimus Docker instaliacijai](https://docs.docker.com/engine/install/)). Rekomenduojama `Debian Bookworm 12 (stable)`. 


#### Kiti

| Komponentas | Min versija | URL |
| ---|-|-|
| Docker | 27.0.3 | [Link](https://docs.docker.com/engine/install/)

Papildomi įrankiai naudojami instaliuojant: [make](https://www.gnu.org/software/make/manual/make.html), [git](https://git-scm.com/download/linux).

### Tinklas

- Pasiekiami port'ai: `443`, `80`.
- Diegimui prisijungimas per ssh: portas `22`

### Vartotojas

Vartotojas kuris diegia, turi turėti `root` teises.

## Prieš diegiant

1. Prisijunkite prie serverio su ssh

2. Patikrinkite ar visi reikalingi komponentai veikia mašinoje:

```bash
    ## Docker
    docker run hello-world
    docker system info
    ## Kiti komponentai
    make --version
    git --version
```   
 
## Diegimas

1. Prisijunkite prie serverio su ssh

1. Parsisiųskite diegimo skriptus (ši git repositorija):

    `git clone https://github.com/airenas/batch-transcriber.git`

    `cd batch-transcriber/deploy/dipolis`

    Docker diegimo skriptai yra direktorijoje *batch-transcriber/deploy/dipolis*.

1. Pasirinkite diegimo versiją:

    `git checkout <VERSIJA>`
    
    `<VERSIJA>` pateiks VDU

1. Paruoškite konfigūracinį diegimo failą *Makefile.options*:

    `cp Makefile.options.template Makefile.options`

1. Sukonfigūruokite *Makefile.options*:

    | Parametras | Priva-lomas | Paskirtis | Pvz |
    |------------------|-----|-----------------------------------|------------------|
    | *data_dir* | + | Direktorija, kur  įkeliami audio failai ir saugomi rezultatai | /data | 
    | *postgres_pass* | + | DB serviso slaptažodis. Nurodykite slaptažodį, kurį servisai naudos prisijungimui prie DB. Pvz.: sugeneruokite su `pwgen 50 1` ||

2. Instaliuokite

    `make install`

    Skriptas parsiųs ir paleis reikalingus docker conteinerius.

## Patikrinimas

1. Patikrinkite ar visi servisai veikia su *docker compose*: `docker compose ps`. Servisai `postgres`, `worker`, `keeper`, `upload-gui` turi būti *Up* būsenoje.

1. Atidarykite URL naršyklėje: *<host/audio-upload/*. Turi atsidaryti failų pateikimo puslapis.

## Servisų sustabdymas/valdymas

Servisai valdomi su *docker compose* komanda:

```bash
    ## Servisų sustabdymas
    docker compose stop
    ##Paleidimas
    docker compose up -d
```

## Duomenų atnaujinimas

1. Atnaujinus duomenis, bus pakeista ir ši repositorija su nuorodomis į naujus docker konteinerius. Patikrinkite, kad turite naujausius skriptus:

    `git pull`

1. Pasirinkite norimą versiją:

    `git checkout <VERSIJA>`

    Versija turi priskirtą *git* žymą. Galimas versijas galite sužinoti su komanda: `git tag`.

1. Jei pasikeitė konfigūracija - atnaujinkite `Makefile.options`

1. Atnaujinkite servisus - pašalinkite ir sudiekite iš naujo:

```bash
    docker compose down
    make install
```

## Pašalinimas

```bash
    docker compose down
```
