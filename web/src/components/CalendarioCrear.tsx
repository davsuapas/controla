import * as React from 'react';
import { useNavigate } from 'react-router';
import useNotifications from '../hooks/useNotifications/useNotifications';
import PageContainer from './PageContainer';
import { NetErrorControlado } from '../net/interceptor';
import { api } from '../api/fabrica';
import { logError } from '../error';
import CalendarioForm, {
  validaCalendario,
  type FormFieldValue,
  type CalendarioFormState,
} from './CalendarioForm';
import { Calendario } from '../modelos/calendario';

const INITIAL_FORM_VALUES: Partial<CalendarioFormState['values']> = {
  nombre: '',
  descripcion: '',
};

export default function CalendarioCrear() {
  const navegar = useNavigate();
  const notifica = useNotifications();

  const [formState, setFormState] = React.useState<CalendarioFormState>({
    values: INITIAL_FORM_VALUES,
    errors: {},
  });

  const handleFormFieldChange = React.useCallback(
    (name: keyof CalendarioFormState['values'], value: FormFieldValue) => {
      setFormState((currentState) => {
        const newFormValues = { ...currentState.values, [name]: value };
        const { issues } = validaCalendario(newFormValues);
        const fieldError = issues.find(issue => issue.path[0] === name)?.message;

        return {
          ...currentState,
          values: newFormValues,
          errors: { ...currentState.errors, [name]: fieldError },
        };
      });
    },
    [],
  );

  const handleFormReset = React.useCallback(() => {
    setFormState({ values: INITIAL_FORM_VALUES, errors: {} });
  }, []);

  const handleFormSubmit = React.useCallback(
    async (formValues: Partial<CalendarioFormState['values']>) => {
      const { issues } = validaCalendario(formValues);

      if (issues.length > 0) {
        const newErrors = Object.fromEntries(
          issues.map(issue => [issue.path[0], issue.message])
        );
        setFormState(prev => ({ ...prev, errors: newErrors }));

        notifica.show('Imposible crear el calendario. Corrija los errores', {
          severity: 'warning',
          autoHideDuration: 5000,
        });
        return;
      }

      setFormState(prev => ({ ...prev, errors: {} }));

      formValues.id = 0;

      try {
        await api().calendar.crearCalendario(formValues as Calendario);

        notifica.show('Calendario creado satisfactoriamente', {
          severity: 'success',
          autoHideDuration: 5000,
        });

        navegar('/calendarios');
      } catch (error) {
        if (error instanceof NetErrorControlado) {
          return;
        }

        logError('calendario-crear.crear', error);

        notifica.show('Error inesperado al crear el calendario', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    }, [notifica, navegar]);

  return (
    <PageContainer
      title="Nuevo calendario"
      breadcrumbs={[{ title: 'Calendarios', path: '/calendarios' }, { title: 'Nuevo' }]}
    >
      <CalendarioForm
        formState={formState}
        onFieldChange={handleFormFieldChange}
        onSubmit={handleFormSubmit}
        onReset={handleFormReset}
        submitButtonLabel="CREAR"
      />
    </PageContainer>
  );
}