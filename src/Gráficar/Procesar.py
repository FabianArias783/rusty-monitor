import os
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

# Obtener la carpeta donde está este script
base_dir = os.path.dirname(os.path.abspath(__file__))

# Ruta relativa al CSV dentro del repo
csv_path = os.path.join(base_dir, 'metrics.csv')

# Leer el archivo CSV
df = pd.read_csv(csv_path)

# Convertir la columna 'Hora' a datetime
df['Hora'] = pd.to_datetime(df['Hora'], errors='coerce')

# Procesar 'Uso_CPU' (calcular promedio si es necesario)
def extraer_cpu_promedio(cpu_data):
    import re
    valores = re.findall(r'\d+\.\d+', cpu_data) if pd.notnull(cpu_data) else []
    if valores:
        valores_float = [float(v) for v in valores]
        return sum(valores_float) / len(valores_float)
    return None

df['Uso_CPU'] = df['Uso_CPU'].apply(extraer_cpu_promedio)

# Limpiar otras columnas numéricas
columnas_a_convertir = ['Memoria_total', 'Memoria_usada']
for col in columnas_a_convertir:
    df[col] = df[col].str.replace('MB', '', regex=False).str.strip()
    df[col] = pd.to_numeric(df[col], errors='coerce')

# Eliminar filas inválidas
df = df.dropna(subset=['Hora', 'Uso_CPU'] + columnas_a_convertir)

# Configurar estilo visual
sns.set(style="whitegrid")

# Generar gráficas
for col in ['Uso_CPU'] + columnas_a_convertir:
    plt.figure(figsize=(10, 5))
    sns.lineplot(x='Hora', y=col, data=df, linewidth=2, color='blue', label=col)
    plt.scatter(df['Hora'], df[col], color='red', s=50, zorder=3, label='Datos')
    plt.xlabel('Hora')
    plt.ylabel(col)
    plt.title(f'{col} a lo largo del tiempo')
    plt.legend()
    plt.tight_layout()
    plt.show()
