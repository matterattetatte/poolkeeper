import { SupabaseClient } from "@supabase/supabase-js";

declare global {
  interface Window {
    supabase: SupabaseClient
  }


//   TODO: FIX!!
//  export type Guest = Database['public']['Tables']['guests']['Row'];
}

export {};
