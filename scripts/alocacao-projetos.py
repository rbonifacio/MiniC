import random

random.seed(20260407)

projetos = [
    "Projeto 1", "Projeto 2", "Projeto 3", "Projeto 4",
        "Projeto 5", "Projeto 6", "Projeto 7", "Projeto 8", "Project 9", "Project 10"]




grupos = ["Grupo 1", "Grupo 2", "Grupo 3", "Grupo 4", "Grupo 5", "Grupo 6"]

random.shuffle(projetos)

projetos_alocados = projetos[:6]


alocacao = dict(zip(grupos, projetos_alocados))

print("Alocacao")
for grupo, projeto in alocacao.items():
    print(f"{projeto} -> {grupo}")

