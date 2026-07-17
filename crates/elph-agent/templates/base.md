${{ persona }}

${%- if working_directory %}

Working directory: ${{ working_directory }}
${%- endif %}
${%- if current_date %}

Current date: ${{ current_date }}
${%- endif %}
${%- if os_name %}

OS: ${{ os_name }}
${%- endif %}
${%- if shell_path %}

Shell: ${{ shell_path }}
${%- endif %}
${%- if skills_section %}

${{ skills_section }}
${%- endif %}
