-- Enforce required publish metadata for versions exposed to the updater.
-- Existing dirty rows are removed first so the NOT NULL constraints can be applied safely.

DELETE FROM public.versions
WHERE platform IS NULL
   OR url IS NULL
   OR hash IS NULL
   OR file_size IS NULL
   OR pub_date IS NULL;

ALTER TABLE public.versions
ALTER COLUMN platform SET NOT NULL,
ALTER COLUMN url SET NOT NULL,
ALTER COLUMN hash SET NOT NULL,
ALTER COLUMN file_size SET NOT NULL;