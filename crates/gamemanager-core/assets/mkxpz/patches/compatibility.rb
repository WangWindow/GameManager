# Global mkxp-z compatibility patch.
# It is loaded before every game launched by GameManager.

if defined?($RGSS_SCRIPTS) && $RGSS_SCRIPTS
  $RGSS_SCRIPTS.delete_if do |script|
    name = script[1].to_s.downcase
    case name
    when "steam_acheivement", "steam_achievement"
      puts "[mkxp patch] Disabled incompatible script: #{script[1]}"
      true
    else
      false
    end
  end
end

unless defined?(SteamUserStatsLite)
  class SteamUserStatsLite
    def self.instance
      @instance ||= new
    end

    def self.restart_app_if_necessary(*args)
      false
    end

    def initialize
    end

    def initted?
      false
    end

    def shutdown
      nil
    end

    def update
      nil
    end

    def is_subscribed
      false
    end

    def is_dlc_installed(*args)
      false
    end

    def request_current_stats
      false
    end

    def get_stat_int(*args)
      0
    end

    def get_stat_float(*args)
      0.0
    end

    def set_stat(*args)
      false
    end

    def update_avg_rate_stat(*args)
      false
    end

    def get_achievement(*args)
      false
    end

    def set_achievement(*args)
      false
    end

    def clear_achievement(*args)
      false
    end

    def get_achievement_and_unlock_time(*args)
      nil
    end

    def get_achievement_display_attribute(*args)
      nil
    end

    def indicate_achievement_progress(*args)
      false
    end

    def get_num_achievements
      0
    end

    def get_achievement_name(*args)
      nil
    end

    def reset_all_stats(*args)
      false
    end

    def method_missing(name, *args, &block)
      puts "[mkxp patch] Ignored SteamUserStatsLite##{name}"
      false
    end

    def respond_to_missing?(name, include_private = false)
      true
    end
  end

  puts "[mkxp patch] Installed SteamUserStatsLite compatibility stub"
end
