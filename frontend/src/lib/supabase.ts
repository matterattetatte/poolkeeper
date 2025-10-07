import { createClient } from '@supabase/supabase-js'
import { Database } from './database.types'

const supabase = createClient<Database>(
  'https://feppfcqtlebctpjwtlmz.supabase.co',
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImZlcHBmY3F0bGViY3Rwand0bG16Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzUzMTA2NDAsImV4cCI6MjA1MDg4NjY0MH0.i6tOkKInsGT2in-iExUzcO89ZcMtWJ5fvo0WOdPDsbU',
)

window.supabase = supabase

export default supabase
