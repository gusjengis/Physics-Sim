import matplotlib.pyplot as plt
import numpy as np
import pandas as pd



def bond_behavior_plots(df,experiment,kn_ks):
    plt.close('all')
    fig = plt.figure(figsize=(9,10), constrained_layout=True)
    fig.suptitle(f'Experiment {experiment}, kn/ks = {kn_ks}', fontsize=16)
    gs = fig.add_gridspec(4, 3)

        # X Position
    ax1 = fig.add_subplot(gs[0, 0])
    ax1.set_ylabel('Position (m)')
    ax1.plot(df["Timestamp"], df["X Position"])
    ax1.set_title('X')
    
    # Y Position
    ax2 = fig.add_subplot(gs[0, 1])
    ax2.sharex(ax1)
    ax2.plot(df["Timestamp"], df["Y Position"])
    ax2.set_title('Y')
    
    # Rotation
    ax3 = fig.add_subplot(gs[0, 2])
    ax3.sharex(ax1)
    ax3.plot(df["Timestamp"], df["Rotation"])
    ax3.set_title('Rotation (degrees)')
    
    # X Velocity
    ax4 = fig.add_subplot(gs[1, 0])
    ax4.sharex(ax1)
    ax4.set_ylabel('Velocity (m/s)')
    ax4.plot(df["Timestamp"], df["X Velocity"])
    ax4.set_xlabel('Cycle Time (s)')
    
    # Y Velocity
    ax5 = fig.add_subplot(gs[1, 1])
    ax5.sharex(ax1)
    ax5.plot(df["Timestamp"], df["Y Velocity"])
    ax5.set_xlabel('Cycle Time (s)')
    
    # Rotation Velocity
    ax6 = fig.add_subplot(gs[1, 2])
    ax6.sharex(ax1)
    ax6.plot(df["Timestamp"], df["Rotation Velocity"])
    ax6.set_xlabel('Cycle Time (s)')

    # Contact Normal direction
    # ax7 = fig.add_subplot(gs[2,1])
    # ax7.sharex(ax1)
    # ax7.plot(df["Timestamp"], df["c_normal_x_dir"]-df["c_normal_x_dir"].iloc[0], label='x')
    # ax7.plot(df["Timestamp"], df["c_normal_y_dir"]-df["c_normal_y_dir"].iloc[0], label='y')
    # ax7.set_title('Change in Contact Normal Direction')
    # # ax7.plot(df["Timestamp"], df["c_normal_x_dir"], label='x')
    # # ax7.plot(df["Timestamp"], df["c_normal_y_dir"], label='y')
    # # ax7.set_title('Contact Normal Direction')
    # ax7.set_xlabel('Cycle Time (s)')
    # ax7.legend(bbox_to_anchor=(0, -0.3, 1., .05),ncols=2, mode="expand")

    # Contact Normal Force
    ax8 = fig.add_subplot(gs[3,0])
    ax8.sharex(ax1)
    ax8.plot(df["Timestamp"], df["Data1"])
    ax8.set_xlabel('Cycle Time (s)')
    ax8.set_title('Contact Normal Force')

    # Contact Shear Force
    ax9 = fig.add_subplot(gs[3,1])
    ax9.sharex(ax1)
    ax9.plot(df["Timestamp"], df["Data2"])
    ax9.set_xlabel('Cycle Time (s)')
    ax9.set_title('Contact Shear Force')

    # Contact Moment
    ax10 = fig.add_subplot(gs[3,2])
    ax10.sharex(ax1)
    ax10.plot(df["Timestamp"], df["Data3"])
    ax10.set_xlabel('Cycle Time (s)')
    ax10.set_title('Contact Moment')

    ax1.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    ax2.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    ax3.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    ax4.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    ax5.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    ax6.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    # ax7.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    ax8.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    ax9.ticklabel_format(style='sci', axis='y', scilimits=(0,0))
    ax10.ticklabel_format(style='sci', axis='y', scilimits=(0,0))

    fig2, ax = plt.subplots()
    ax.plot(df["X Position"], df["Y Position"], linewidth=0.5, c='black')
    pos = ax.scatter(df["X Position"], df["Y Position"], s=10, c=df["Timestamp"], cmap='coolwarm')
    fig2.colorbar(pos, ax=ax)
    ax.set_xlabel('X Position (m)')
    ax.set_ylabel('Y Position (m)')
    ax.set_title('Ball position')