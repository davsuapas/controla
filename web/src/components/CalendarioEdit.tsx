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
import CalendarioForm, {
  validaCalendario,
  type FormFieldValue,
  type CalendarioFormState,
} from './CalendarioForm';
import { Calendario } from '../modelos/calendario';

function CalendarioEditForm({
  initialValues,
  onSubmit,
}: {
  initialValues: Partial<CalendarioFormState['values']>;
  onSubmit: (formValues: Partial<CalendarioFormState['values']>) => Promise<void>;
}) {
  const navegar = useNavigate();
  const notifica = useNotifications();

  const [formState, setFormState] = React.useState<CalendarioFormState>({
    values: initialValues,
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
    setFormState({ values: initialValues, errors: {} });
  }, [initialValues]);

  const handleFormSubmit = React.useCallback(async (formValues: Partial<CalendarioFormState['values']>) => {
    const { issues } = validaCalendario(formValues);

    if (issues.length > 0) {
      const newErrors = Object.fromEntries(
        issues.map(issue => [issue.path[0], issue.message])
      );
      setFormState(prev => ({ ...prev, errors: newErrors }));

      notifica.show('Imposible actualizar el calendario. Corrija los errores', {
        severity: 'warning',
        autoHideDuration: 5000,
      });
      return;
    }

    setFormState(prev => ({ ...prev, errors: {} }));

    try {
      await onSubmit(formValues);

      notifica.show('Calendario actualizado satisfactoriamente', {
        severity: 'success',
        autoHideDuration: 5000,
      });

      navegar('/calendarios');
    } catch (error) {
      if (error instanceof NetErrorControlado) return;

      logError('calendario-editar.actualizar', error);

      notifica.show('Error inesperado al actualizar el calendario', {
        severity: 'error',
        autoHideDuration: 5000,
      });
    }
  }, [notifica, navegar, onSubmit]);

  return (
    <CalendarioForm
      formState={formState}
      onFieldChange={handleFormFieldChange}
      onSubmit={handleFormSubmit}
      onReset={handleFormReset}
      submitButtonLabel="ACTUALIZAR"
    />
  );
}

export default function CalendarioEdit() {
  const { id } = useParams<{ id: string }>();
  const isMounted = useIsMounted();

  const [calendario, setCalendario] = React.useState<Calendario | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);
  const [error, setError] = React.useState<Error | null>(null);

  React.useEffect(() => {
    const loadData = async () => {
      setError(null);
      setIsLoading(true);
      try {
        const showData = await api().calendar.calendario(id ?? '');
        if (isMounted.current) setCalendario(showData);
      } catch (err) {
        if (!(err instanceof NetErrorControlado)) {
          logError('calendario-editar.cargar', err);
          setError(Error('Error inesperado al cargar el calendario'));
        }
      } finally {
        if (isMounted.current) setIsLoading(false);
      }
    };
    loadData();
  }, [id, isMounted]);

  const handleSubmit = React.useCallback(
    async (formValues: Partial<CalendarioFormState['values']>) => {
      await api().calendar.actualizarCalendario(formValues as Calendario);
    },
    [],
  );

  const renderContent = () => {
    if (isLoading) {
      return <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}><CircularProgress /></Box>;
    }
    if (error) {
      return <Alert severity="error">{error.message}</Alert>;
    }
    if (calendario) {
      return <CalendarioEditForm initialValues={calendario} onSubmit={handleSubmit} />;
    }
    return null;
  };

  return (
    <PageContainer
      title={`Edición del calendario: ${calendario?.nombre ?? id}`}
      breadcrumbs={[{ title: 'Calendarios', path: '/calendarios' }, { title: 'Edición' }]}
    >
      <Box sx={{ flex: 1 }}>{renderContent()}</Box>
    </PageContainer>
  );
}