from django.forms import forms
from django.forms import CharField, TextInput


class MyForm(forms.Form):
    def __init__(self, context):
        super().__init__(context)

        for talent in context['talents']:
            talent_id = talent[0]
            talent_display_name = talent[1]
            talent_max_value = talent[2]

            self.fields.update({
                f'talents_{talent_id}': CharField(
                    required=False,
                    label=talent_display_name,
                    widget=TextInput(
                        attrs={
                            'type': 'number',
                            'min': "0",
                            'step': "1",
                            'max': talent_max_value,
                            'value': 0,
                            'id': talent_id,
                            'label': talent_display_name,
                            'class': 'w-25',
                        },

                    )
                )
            })

