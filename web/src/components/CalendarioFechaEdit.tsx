import * as React from 'react';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import CircularProgress from '@mui/material/CircularProgress';
import { useNavigate, useParams } from 'react-router';
import useNotifications from '../hooks/useNotifications/useNotifications';
import PageContainer from './PageContainer';
import { NetErrorControlado } from '../net/interceptor';
import { api } from '../api/fabrica';
import { logError } from '../error';
import { useIsMounted } from '../hooks/useComponentMounted';
import CalendarioFechaForm, {
  validaCalendarioFecha,
  type FormFieldValue,
  type CalendarioFechaFormState,
} from './CalendarioFechaForm';
import { CalendarioFecha } from '../modelos/calendario';

function CalendarioFechaEditForm({
  initialValues,
  onSubmit,
  onBack,
}: {
  initialValues: Partial<CalendarioFechaFormState['values']>;
  onSubmit: (formValues: Partial<CalendarioFechaFormState['values']>) => Promise<void>;
  onBack: () => void;
}) {
  const notifica = useNotifications();

  const [formState, setFormState] = React.useState<CalendarioFechaFormState>({
    values: initialValues,
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
    setFormState({ values: initialValues, errors: {} });
  }, [initialValues]);

  const handleFormSubmit = React.useCallback(async (formValues: Partial<CalendarioFechaFormState['values']>) => {
    const { issues } = validaCalendarioFecha(formValues);

    if (issues.length > 0) {
      const newErrors = Object.fromEntries(
        issues.map(issue => [issue.path[0], issue.message])
      );
      setFormState(prev => ({ ...prev, errors: newErrors }));

      notifica.show('Imposible actualizar la fecha. Corrija los errores', {
        severity: 'warning',
        autoHideDuration: 5000,
      });
      return;
    }

    setFormState(prev => ({ ...prev, errors: {} }));

    try {
      await onSubmit(formValues);

      notifica.show('Fecha actualizada satisfactoriamente', {
        severity: 'success',
        autoHideDuration: 5000,
      });

    } catch (error) {
      if (error instanceof NetErrorControlado) return;
      logError('calendario-fecha-editar.actualizar', error);
      notifica.show('Error inesperado al actualizar la fecha', {
        severity: 'error',
        autoHideDuration: 5000,
      });
    }
  }, [notifica, onSubmit]);

  return (
    <CalendarioFechaForm
      formState={formState}
      onFieldChange={handleFormFieldChange}
      onSubmit={handleFormSubmit}
      onReset={handleFormReset}
      submitButtonLabel="ACEPTAR"
      onBack={onBack}
    />
  );
}

export default function CalendarioFechaEdit() {
  const { id: calendarioId, fechaId } = useParams<{ id: string, fechaId: string }>();
  const isMounted = useIsMounted();
  const navegar = useNavigate();

  const [fecha, setFecha] = React.useState<CalendarioFecha | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);
  const [error, setError] = React.useState<Error | null>(null);

  React.useEffect(() => {
    const loadData = async () => {
      setError(null);
      setIsLoading(true);
      try {
        const showData = await api().calendar.fecha(Number(fechaId));
        if (isMounted.current) setFecha(showData);
      } catch (err) {
        if (!(err instanceof NetErrorControlado)) {
          logError('calendario-fecha-editar.cargar', err);
          if (isMounted.current) {
            setError(Error('Error inesperado al cargar la fecha del calendario'));
          }
        }
      } finally {
        if (isMounted.current) setIsLoading(false);
      }
    };
    loadData();
  }, [fechaId, isMounted]);

  const handleSubmit = React.useCallback(
    async (formValues: Partial<CalendarioFechaFormState['values']>) => {
      await api().calendar.actualizarFecha(new CalendarioFecha(formValues));
      navegar(`/calendarios/${calendarioId}/fechas`);
    },
    [calendarioId, navegar],
  );

  const handleBack = React.useCallback(() => {
    navegar(`/calendarios/${calendarioId}/fechas`);
  }, [navegar, calendarioId]);

  const renderContent = () => {
    if (isLoading) {
      return <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}><CircularProgress /></Box>;
    }
    if (error) {
      return <Alert severity="error">{error.message}</Alert>;
    }
    if (fecha) {
      return <CalendarioFechaEditForm initialValues={fecha} onSubmit={handleSubmit} onBack={handleBack} />;
    }
    return null;
  };

  return (
    <PageContainer
      title={`Edición de fecha: ${fechaId}`}
      breadcrumbs={[{ title: 'Calendarios', path: '/calendarios' }, { title: 'Fechas', path: `/calendarios/${calendarioId}/fechas` }, { title: 'Edición' }]}
    >
      <Box sx={{ flex: 1 }}>{renderContent()}</Box>
    </PageContainer>
  );
}