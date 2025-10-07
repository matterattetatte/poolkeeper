import { createClient } from '@supabase/supabase-js'
import { Database } from './database.types'

const supabase = createClient<Database>(
  'https://haoyjqowxacradtsgksc.supabase.co',
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imhhb3lqcW93eGFjcmFkdHNna3NjIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NTIwNjEwODAsImV4cCI6MjA2NzYzNzA4MH0.rwGHZZQPnGnAElX8IvfYTln9Cs1uhuYOYShR8eLetMc',
)

window.supabase = supabase

export default supabase
