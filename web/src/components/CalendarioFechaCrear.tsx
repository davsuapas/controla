import * as React from 'react';
import { useNavigate, useParams } from 'react-router';
import useNotifications from '../hooks/useNotifications/useNotifications';
import PageContainer from './PageContainer';
import { NetErrorControlado } from '../net/interceptor';
import { api } from '../api/fabrica';
import { logError } from '../error';
import CalendarioFechaForm, {
  validaCalendarioFecha,
  type FormFieldValue,
  type CalendarioFechaFormState,
} from './CalendarioFechaForm';
import { CalendarioFecha, TipoCalendarioFecha } from '../modelos/calendario';
import dayjs from 'dayjs';

const INITIAL_FORM_VALUES: Partial<CalendarioFechaFormState['values']> = {
  fechaInicio: dayjs(),
  fechaFin: dayjs(),
  tipo: TipoCalendarioFecha.Baja,
};

export default function CalendarioFechaCrear() {
  const { id: calendarioId } = useParams<{ id: string }>();
  const navegar = useNavigate();
  const notifica = useNotifications();

  const [formState, setFormState] = React.useState<CalendarioFechaFormState>({
    values: INITIAL_FORM_VALUES,
    errors: {},
  });

  const handleFormFieldChange = React.useCallback(
    (name: keyof CalendarioFechaFormState['values'], value: FormFieldValue) => {
      setFormState((currentState) => {
        const newFormValues = { ...currentState.values, [name]: value };
        const { issues } = validaCalendarioFecha(newFormValues);
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
    async (formValues: Partial<CalendarioFechaFormState['values']>) => {
      const { issues } = validaCalendarioFecha(formValues);

      if (issues.length > 0) {
        const newErrors = Object.fromEntries(
          issues.map(issue => [issue.path[0], issue.message])
        );
        setFormState(prev => ({ ...prev, errors: newErrors }));

        notifica.show('Imposible crear la fecha. Corrija los errores', {
          severity: 'warning',
          autoHideDuration: 5000,
        });
        return;
      }

      setFormState(prev => ({ ...prev, errors: {} }));

      try {
        const fechaData = {
          ...formValues,
          id: 0,
          calendario: Number(calendarioId),
        }
        await api().calendar.crearFecha(new CalendarioFecha(fechaData));

        notifica.show('Fecha creada satisfactoriamente', {
          severity: 'success',
          autoHideDuration: 5000,
        });

        navegar(`/calendarios/${calendarioId}/fechas`);
      } catch (error) {
        if (error instanceof NetErrorControlado) {
          return;
        }
        logError('calendario-fecha-crear.crear', error);
        notifica.show('Error inesperado al crear la fecha', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    }, [notifica, navegar, calendarioId]);

  const handleBack = React.useCallback(() => {
    navegar(`/calendarios/${calendarioId}/fechas`);
  }, [navegar, calendarioId]);

  return (
    <PageContainer
      title="Nueva fecha de calendario"
      breadcrumbs={[{ title: 'Calendarios', path: '/calendarios' }, { title: 'Fechas', path: `/calendarios/${calendarioId}/fechas` }, { title: 'Nueva' }]}
    >
      <CalendarioFechaForm
        formState={formState}
        onFieldChange={handleFormFieldChange}
        onSubmit={handleFormSubmit}
        onReset={handleFormReset}
        submitButtonLabel="ACEPTAR"
        onBack={handleBack}
      />
    </PageContainer>
  );
}